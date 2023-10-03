use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use prost::Message;

use yharnam::*;

#[derive(Debug, PartialEq, Eq)]
pub enum PlanStep {
    Line(String),
    Tags(Vec<String>),
    Option(String),
    Select(u32),
    Command(String),
    Stop,
}

impl PlanStep {
    fn new(line: &str) -> Self {
        let mut split_line = line.splitn(2, ": ");
        match split_line.next() {
            Some("line") => Self::Line(split_line.next().unwrap().to_owned()),
            Some("tags") => Self::Tags(
                split_line
                    .collect::<String>()
                    .split(" ")
                    .map(|item| item.to_owned())
                    .collect::<Vec<String>>(),
            ),
            Some("option") => Self::Option(split_line.next().unwrap().to_owned()),
            Some("select") => {
                let index: u32 = split_line.next().and_then(|s| s.parse().ok()).unwrap();
                if index < 1 {
                    panic!("Select index must be 1 or greater.");
                }
                Self::Select(index - 1)
            }
            Some("command") => Self::Command(split_line.next().unwrap().to_owned()),
            Some("stop") => Self::Stop,
            Some(step) => panic!(
                "Could not parse test plan step \"{}\" in line \"{}\"",
                step, line
            ),
            None => panic!("Could not parse test plan step in line \"{}\"", line),
        }
    }
}

pub struct TestPlan {
    steps: Vec<PlanStep>,
    next_step_index: usize,
    options: Vec<String>,
}

impl TestPlan {
    pub fn load(plan_path: &Path) -> io::Result<Self> {
        let plan_text = fs::read_to_string(plan_path)?;
        let steps: Vec<_> = plan_text
            .lines()
            .map(|line| line.trim_start())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| PlanStep::new(line))
            .collect();

        Ok(Self {
            steps,
            next_step_index: 0,
            options: Vec::new(),
        })
    }

    // TODO: Can we just return the step and potential options from this?
    pub fn next(&mut self) {
        // step through the test plan until we hit an expectation to
        // see a line, option, or command. specifically, we're waiting
        // to see if we got a Line, Select, Command or Assert step
        // type.

        let prev_step = match self.next_step_index {
            i if i > 0 && i <= self.steps.len() => Some(&self.steps[i as usize - 1]),
            _ => None,
        };
        if let Some(PlanStep::Select(_)) = prev_step {
            // our previously-notified task was to select an option.
            // we've now moved past that, so clear the list of expected
            // options.
            self.options.clear();
        }

        while self.next_step_index <= self.steps.len() {
            let current_step = self.steps.get(self.next_step_index);

            self.next_step_index += 1;

            match current_step {
                Some(PlanStep::Option(option)) => {
                    self.options.push(option.clone());
                    continue;
                }
                _ => return,
            }
        }

        // We've fallen off the end of the test plan step list. We
        // expect a stop here.
        return;
    }

    pub fn get_current_step(&self) -> Option<&PlanStep> {
        match self.next_step_index {
            0 => None,
            i if i <= self.steps.len() => Some(&self.steps[self.next_step_index as usize - 1]),
            _ => Some(&PlanStep::Stop),
        }
    }
}

pub struct PlanRunner {
    vm: VirtualMachine,
    string_table: Vec<LineInfo>,
    metadata_table: Vec<MetadataInfo>,
    plan: TestPlan,
    locale: String,
}

impl PlanRunner {
    pub fn new(yarn_path: &str) -> Self {
        let yarn_path = Path::new(yarn_path);

        let proto_path = yarn_path.with_extension("yarnc");

        // Read the file's bytes and load a Program.
        let proto_data = fs::read(&proto_path).unwrap();
        let program = Program::decode(&*proto_data).unwrap();

        // Load LineInfos from a csv file.
        let mut csv_path = proto_path.clone();
        csv_path.set_file_name(format!(
            "{}-Lines.csv",
            csv_path.file_stem().unwrap().to_str().unwrap()
        ));
        let string_table: Vec<LineInfo> = csv::Reader::from_path(csv_path)
            .unwrap()
            .deserialize()
            .map(|result| result.unwrap())
            .collect();

        // Load Metadata from a csv file.
        let mut csv_path = proto_path;
        csv_path.set_file_name(format!(
            "{}-Metadata.csv",
            csv_path.file_stem().unwrap().to_str().unwrap()
        ));
        let metadata_table: Vec<MetadataInfo> = csv::ReaderBuilder::new()
            .flexible(true)
            .from_path(csv_path)
            .unwrap()
            .deserialize()
            .map(|result| result.unwrap())
            .collect();

        let mut vm = VirtualMachine::new(program);
        vm.library.insert(
            "assert".to_string(),
            FunctionInfo::new(1, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
                if !parameters[0].as_bool() {
                    assert!(false, "Assertion failed");
                }
            }),
        );

        let plan_path = yarn_path.with_extension("testplan");
        let plan = TestPlan::load(&plan_path).unwrap();

        Self {
            vm,
            string_table,
            metadata_table,
            plan,
            locale: "en".to_string(),
        }
    }

    /// Gets a mutable ref to the [VirtualMachine] so that it can be configured
    /// for the test (for instance to seed the RNG).
    pub fn get_vm(&mut self) -> &mut VirtualMachine {
        &mut self.vm
    }

    fn get_tags_for_line(&self, line: &Line) -> Vec<String> {
        self.metadata_table
            .iter()
            .find(|metadata_info| metadata_info.id == line.id)
            .map(|metadata_info| &metadata_info.tags)
            .cloned()
            .unwrap_or_else(|| Vec::new())
    }

    fn get_composed_text_for_line(&self, line: &Line) -> String {
        let mut line_text = self
            .string_table
            .iter()
            .find(|line_info| line_info.id == line.id)
            .map(|line_info| &line_info.text)
            .unwrap()
            .clone();
        for (i, substitution) in line.substitutions.iter().enumerate() {
            line_text = line_text.replacen(&format!("{{{}}}", i), &substitution, 1);
        }

        yharnam::expand_format_functions(&line_text, &self.locale)
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.vm.set_node("Start")?;

        loop {
            match self.vm.continue_dialogue()? {
                SuspendReason::Nop => {}
                SuspendReason::Line(line) => {
                    // Assert that the test plan expects this line.
                    self.plan.next();

                    let plan_step = self.plan.get_current_step().unwrap();
                    let line_text = self.get_composed_text_for_line(&line);

                    assert!(
                        matches!(plan_step, PlanStep::Line(plan_text) if *plan_text == line_text),
                        "[{}] Expected the line {:?}, got \"{}\"",
                        self.plan.next_step_index,
                        plan_step,
                        line_text
                    );

                    let tags = self.get_tags_for_line(&line);
                    if !tags.is_empty() {
                        self.plan.next();

                        let plan_step = self.plan.get_current_step().unwrap();
                        if let PlanStep::Tags(found_tags) = plan_step {
                            assert!(
                                tags.iter().zip(found_tags).all(|(a, b)| a == b),
                                "[{}] Expected tag step \"{plan_step:?}\" but found tags: {tags:?}",
                                self.plan.next_step_index
                            )
                        } else {
                            panic!(
                                "[{}] Expected tag step with tags: {tags:?}, bot got \"{plan_step:?}\"",
                                self.plan.next_step_index
                            )
                        }

                        assert!(
                            matches!(plan_step, PlanStep::Tags(found_tags) if *found_tags == tags),
                            "[{}] Expected tag step {:?}, got \"{tags:?}\"",
                            self.plan.next_step_index,
                            plan_step,
                        );
                    }
                }
                SuspendReason::Options(options) => {
                    // Assert that the test plan expects these options.
                    self.plan.next();
                    let plan_step = self.plan.get_current_step().unwrap();
                    for (option, plan_option) in options.into_iter().zip(&self.plan.options) {
                        let option_text = self.get_composed_text_for_line(&option.line);
                        assert_eq!(option_text, *plan_option);
                    }
                    match plan_step {
                        PlanStep::Select(i) => self.vm.set_selected_option(*i)?,
                        step => panic!("Expected PlanStep::Select, got {:?}", step),
                    }
                }
                SuspendReason::Command(command) => {
                    // Assert that the test plan expects this command.
                    self.plan.next();
                    let plan_step = self.plan.get_current_step().unwrap();
                    assert!(
                        matches!(plan_step, PlanStep::Command(plan_text) if *plan_text == command),
                        "Expected the command {:?}, got \"{}\"",
                        plan_step,
                        command
                    );
                }
                SuspendReason::NodeChange { .. } => {}
                SuspendReason::DialogueComplete(_) => {
                    // Assert that the test plan expects the end of dialogue.
                    self.plan.next();
                    let plan_step = self.plan.get_current_step().unwrap();
                    assert_eq!(*plan_step, PlanStep::Stop);
                    break;
                }
                SuspendReason::InvalidOption(option_name) => {
                    panic!("Unexpected option name: {}", option_name);
                }
            }
        }

        Ok(())
    }
}
