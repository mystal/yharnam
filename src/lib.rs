use std::collections::HashMap;

use log::*;
use serde::Deserialize;

pub use crate::{
    yarn_proto::Program,
    value::YarnValue,
};

pub mod yarn_proto {
    include!(concat!(env!("OUT_DIR"), "/yarn.rs"));
}

mod value;

#[derive(Debug, Deserialize)]
pub struct Record {
    pub id: String,
    pub text: String,
    pub file: String,
    pub node: String,
    #[serde(rename="lineNumber")]
    pub line_number: u32,
}

#[derive(Debug, Clone)]
pub struct Line {
    pub id: String,
    // substitutions: Vec<String>,
}

impl Line {
    pub fn new(id: String) -> Self {
        Self {
            id,
        }
    }
}

pub struct YarnOption {
    pub line: Line,
    pub id: u32,
    pub destination_node: String,
}

impl YarnOption {
    fn new(line: Line, id: u32, destination_node: String) -> Self {
        Self {
            line,
            id,
            destination_node,
        }
    }
}

pub type ReturningFunction = dyn Fn(&[YarnValue]) -> YarnValue;
pub type Function = dyn Fn(&[YarnValue]);

pub enum YarnFunction {
    Void(&'static Function),
    Returning(&'static ReturningFunction),
}

impl YarnFunction {
    pub fn call(&self, params: &[YarnValue]) -> Option<YarnValue> {
        match self {
            Self::Void(func) => {
                (func)(params);
                None
            }
            Self::Returning(func) => {
                let result = (func)(params);
                Some(result)
            }
        }
    }
}

pub struct FunctionInfo {
    param_count: i8,
    func: YarnFunction,
}

impl FunctionInfo {
    pub fn new(param_count: i8, func: &'static Function) -> Self {
        Self {
            param_count,
            func: YarnFunction::Void(func),
        }
    }

    pub fn new_returning(param_count: i8, func: &'static ReturningFunction) -> Self {
        Self {
            param_count,
            func: YarnFunction::Returning(func),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExecutionState {
    Stopped,
    WaitingOnOptionSelection,
    Suspended,
    Running,
}

pub struct VmState {
    pub current_node_name: String,
    // TODO: Switch back to usize soon.
    pub program_counter: isize,
    pub current_options: Vec<(Line, String)>,
    pub stack: Vec<YarnValue>,
}

impl VmState {
    fn new() -> Self {
        Self {
            current_node_name: String::new(),
            program_counter: 0,
            current_options: Vec::new(),
            stack: Vec::new(),
        }
    }
}

pub struct VirtualMachine {
    pub state: VmState,
    pub variable_storage: HashMap<String, YarnValue>,
    pub library: HashMap<String, FunctionInfo>,

    pub execution_state: ExecutionState,

    pub program: Program,
    // TODO: string_table should live in the client.
    pub string_table: Vec<Record>,
}

impl VirtualMachine {
    pub fn new(program: Program, string_table: Vec<Record>) -> Self {
        let mut library = HashMap::new();
        library.insert(
            "Add".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                parameters[0].add(&parameters[1]).unwrap()
            }),
        );

        library.insert(
            "Minus".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                parameters[0].sub(&parameters[1]).unwrap()
            }),
        );

        library.insert(
            "UnaryMinus".to_string(),
            FunctionInfo::new_returning(1, &|parameters: &[YarnValue]| {
                parameters[0].neg()
            }),
        );

        library.insert(
            "Divide".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                parameters[0].div(&parameters[1]).unwrap()
            }),
        );

        library.insert(
            "Multiply".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                parameters[0].mul(&parameters[1]).unwrap()
            }),
        );

        library.insert(
            "Modulo".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                parameters[0].rem(&parameters[1]).unwrap()
            }),
        );

        library.insert(
            "EqualTo".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                (parameters[0] == parameters[1]).into()
            }),
        );

        library.insert(
            "NotEqualTo".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                (parameters[0] != parameters[1]).into()
            }),
        );

        library.insert(
            "GreaterThan".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                (parameters[0] > parameters[1]).into()
            }),
        );

        library.insert(
            "GreaterThanOrEqualTo".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                (parameters[0] >= parameters[1]).into()
            }),
        );

        library.insert(
            "LessThan".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                (parameters[0] < parameters[1]).into()
            }),
        );

        library.insert(
            "LessThanOrEqualTo".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                (parameters[0] <= parameters[1]).into()
            }),
        );

        library.insert(
            "And".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                (parameters[0].as_bool() && parameters[1].as_bool()).into()
            }),
        );

        library.insert(
            "Or".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                (parameters[0].as_bool() || parameters[1].as_bool()).into()
            }),
        );

        library.insert(
            "Xor".to_string(),
            FunctionInfo::new_returning(2, &|parameters: &[YarnValue]| {
                (parameters[0].as_bool() ^ parameters[1].as_bool()).into()
            }),
        );

        library.insert(
            "Not".to_string(),
            FunctionInfo::new_returning(1, &|parameters: &[YarnValue]| {
                (!parameters[0].as_bool()).into()
            }),
        );

        Self {
            state: VmState::new(),
            variable_storage: HashMap::new(),
            library,
            execution_state: ExecutionState::Stopped,
            program,
            string_table,
        }
    }

    pub fn set_node(&mut self, node_name: &str) -> bool {
        // TODO: Handle error cases.
        // if (Program == null || Program.Nodes.Count == 0) {
        //     throw new DialogueException($"Cannot load node {nodeName}: No nodes have been loaded.");
        // }

        // if (Program.Nodes.ContainsKey(nodeName) == false) {
        //     executionState = ExecutionState.Stopped;
        //     throw new DialogueException($"No node named {nodeName} has been loaded.");
        // }

        // dialogue.LogDebugMessage ("Running node " + nodeName);

        self.state = VmState::new();
        self.state.current_node_name = node_name.to_string();

        self.node_start_handler(&node_name);

        // TODO: Do we wanna do this?
        self.execution_state = ExecutionState::Suspended;

        true
    }

    // TODO: Return the reason why we stopped execution.
    // Either Line, Options, Command, NodeStart?, NodeEnd?, DialogeEnd
    pub fn continue_dialogue(&mut self) {
        // TODO: Handle error cases.
        // if (currentNode == null)
        // {
        //     throw new DialogueException("Cannot continue running dialogue. No node has been selected.");
        // }

        if self.execution_state == ExecutionState::WaitingOnOptionSelection {
            panic!("Cannot continue running dialogue. Still waiting on option selection.");
        }

        // if (lineHandler == null)
        // {
        //     throw new DialogueException($"Cannot continue running dialogue. {nameof(lineHandler)} has not been set.");
        // }

        // if (optionsHandler == null)
        // {
        //     throw new DialogueException($"Cannot continue running dialogue. {nameof(optionsHandler)} has not been set.");
        // }

        // if (commandHandler == null)
        // {
        //     throw new DialogueException($"Cannot continue running dialogue. {nameof(commandHandler)} has not been set.");
        // }

        // if (nodeCompleteHandler == null)
        // {
        //     throw new DialogueException($"Cannot continue running dialogue. {nameof(nodeCompleteHandler)} has not been set.");
        // }

        // if (nodeCompleteHandler == null)
        // {
        //     throw new DialogueException($"Cannot continue running dialogue. {nameof(nodeCompleteHandler)} has not been set.");
        // }

        self.execution_state = ExecutionState::Running;

        // Execute instructions until something forces us to stop
        while let ExecutionState::Running = self.execution_state {
            let current_instruction = {
                let current_node = &self.program.nodes[&self.state.current_node_name];
                current_node.instructions[self.state.program_counter as usize].clone()
            };

            self.run_instruction(current_instruction);

            // TODO: Make run_instruction return more info about what to do next. i.e., don't
            // always increment the program counter, for one.

            self.state.program_counter += 1;

            let instruction_count = if !self.state.current_node_name.is_empty() {
                self.program.nodes[&self.state.current_node_name].instructions.len()
            } else {
                0
            };

            if self.state.program_counter as usize >= instruction_count {
                self.node_complete_handler(&self.state.current_node_name);

                self.execution_state = ExecutionState::Stopped;
                self.state = VmState::new();

                self.dialogue_complete_handler();
                // dialogue.LogDebugMessage ("Run complete.");
            }
        }
    }

    pub fn set_selected_option(&mut self, selected_option_id: u32) {
        let selected_option_id = selected_option_id as usize;

        if self.execution_state != ExecutionState::WaitingOnOptionSelection {
            panic!();
            // throw new DialogueException(@"SetSelectedOption was called, but Dialogue wasn't waiting for a selection.
            // This method should only be called after the Dialogue is waiting for the user to select an option.");
        }

        if selected_option_id >= self.state.current_options.len() {
            panic!();
            // throw new ArgumentOutOfRangeException($"{selectedOptionID} is not a valid option ID (expected a number between 0 and {state.currentOptions.Count-1}.");
        }

        // We now know what number option was selected; push the
        // corresponding node name to the stack
        let destination_node = self.state.current_options[selected_option_id].1.clone();
        self.state.stack.push(YarnValue::Str(destination_node));

        // We no longer need the accumulated list of options; clear it
        // so that it's ready for the next one
        self.state.current_options.clear();

        // We're no longer in the WaitingForOptions state; we are now
        // instead Suspended
        self.execution_state = ExecutionState::Suspended;

        debug!("Selected option: {}", selected_option_id);
    }

    fn run_instruction(&mut self, instruction: yarn_proto::Instruction) {
        use yarn_proto::{
            instruction::OpCode,
            operand::Value,
        };

        let opcode = OpCode::from_i32(instruction.opcode)
            .unwrap();

        debug!("Running {:?} {:?}", opcode, instruction.operands);

        match opcode {
            OpCode::JumpTo => {
                if let Some(Value::StringValue(label)) = &instruction.operands[0].value {
                    self.state.program_counter = self.find_instruction_point_for_label(label) - 1;
                } else {
                    // TODO: Error.
                }
            }
            OpCode::Jump => {
                if let Some(YarnValue::Str(label)) = self.state.stack.last() {
                    self.state.program_counter = self.find_instruction_point_for_label(label) - 1;
                } else {
                    // TODO: Error.
                }
            }
            OpCode::RunLine => {
                // Looks up a string from the string table and passes it to the client as a line.
                if let Some(Value::StringValue(string_key)) = &instruction.operands[0].value {
                    let line = Line::new(string_key.clone());

                    // TODO: Implement substitutions.
                    // The second operand, if provided (compilers prior
                    // to v1.1 don't include it), indicates the number
                    // of expressions in the line. We need to pop these
                    // values off the stack and deliver them to the
                    // line handler.
                    // if instruction.operands.len() > 1 {
                    //     let expressionCount = (int)i.Operands[1].FloatValue;

                    //     let strings = new string[expressionCount];

                    //     for (int expressionIndex = expressionCount - 1; expressionIndex >= 0; expressionIndex--) {
                    //         strings[expressionIndex] = state.PopValue().AsString;
                    //     }

                    //     line.Substitutions = strings;
                    // }

                    let pause = self.line_handler(line);

                    if pause {
                        self.execution_state = ExecutionState::Suspended;
                    }
                } else {
                    // TODO: Handle this error!
                }
            }
            OpCode::RunCommand => {
                // Passes a string to the client as a custom command
                if let Some(Value::StringValue(command)) = &instruction.operands[0].value {
                    // TODO: Implement substitutions.
                    // The second operand, if provided (compilers prior
                    // to v1.1 don't include it), indicates the number
                    // of expressions in the command. We need to pop
                    // these values off the stack and deliver them to
                    // the line handler.
                    // if (i.Operands.Count > 1) {
                    //     var expressionCount = (int)i.Operands[1].FloatValue;

                    //     var strings = new string[expressionCount];

                    //     // Get the values from the stack, and
                    //     // substitute them into the command text
                    //     for (int expressionIndex = expressionCount - 1; expressionIndex >= 0; expressionIndex--) {
                    //         var substitution = state.PopValue().AsString;

                    //         commandText = commandText.Replace("{" + expressionIndex + "}", substitution);
                    //     }
                    // }

                    let pause = self.command_handler(command);

                    if pause {
                        self.execution_state = ExecutionState::Suspended;
                    }
                } else {
                    // TODO: Error.
                }
            }
            OpCode::AddOption => {
                let line = if let Some(Value::StringValue(opt)) = instruction.operands.get(0).and_then(|o| o.value.as_ref()) {
                    // TODO: Implement substitutions.
                    // if instruction.operands.len() > 2 {
                    //     // get the number of expressions that we're
                    //     // working with out of the third operand
                    //     var expressionCount = (int)i.Operands[2].FloatValue;

                    //     var strings = new string[expressionCount];

                    //     // pop the expression values off the stack in
                    //     // reverse order, and store the list of substitutions
                    //     for (int expressionIndex = expressionCount - 1; expressionIndex >= 0; expressionIndex--) {
                    //         string substitution = state.PopValue().AsString;
                    //         strings[expressionIndex] = substitution;
                    //     }

                    //     line.Substitutions = strings;
                    // }
                    Line::new(opt.clone())
                } else {
                    // TODO: Handle error.
                    panic!();
                };
                let node_name = if let Some(Value::StringValue(opt)) = instruction.operands.get(1).and_then(|o| o.value.as_ref()) {
                    opt.clone()
                } else {
                    // TODO: Handle error.
                    panic!();
                };

                self.state.current_options.push((line, node_name));
            }
            OpCode::ShowOptions => {
                // If we have no options to show, immediately stop.
                if self.state.current_options.is_empty() {
                    self.execution_state = ExecutionState::Stopped;
                    self.state = VmState::new();
                    self.dialogue_complete_handler();
                    return;
                }

                // Present the list of options to the user and let them pick
                let mut options = Vec::new();

                for (i, opt) in self.state.current_options.iter().enumerate() {
                    options.push(YarnOption::new(opt.0.clone(), i as u32, opt.1.clone()));
                }

                // We can't continue until our client tell us which option to pick.
                self.execution_state = ExecutionState::WaitingOnOptionSelection;

                // Pass the options set to the client, as well as a delegate for them to call when the
                // user has made a selection
                self.options_handler(options);
            }
            OpCode::PushString => {
                if let Some(Value::StringValue(val)) = &instruction.operands[0].value {
                    self.state.stack.push(YarnValue::Str(val.clone()));
                } else {
                    // TODO: Error: bad operand.
                }
            }
            OpCode::PushFloat => {
                if let Some(Value::FloatValue(val)) = &instruction.operands[0].value {
                    self.state.stack.push(YarnValue::Number(*val));
                } else {
                    // TODO: Error: bad operand.
                }
            }
            OpCode::PushBool => {
                if let Some(Value::BoolValue(val)) = &instruction.operands[0].value {
                    self.state.stack.push(YarnValue::Bool(*val));
                } else {
                    // TODO: Error: bad operand.
                }
            }
            OpCode::PushNull => {
                self.state.stack.push(YarnValue::Null);
            },
            OpCode::JumpIfFalse => {
                // Jump to a named label if the value on the top of the stack
                // evaluates to the boolean value 'false'.
                if let Some(val) = self.state.stack.last() {
                    if !val.as_bool() {
                        if let Some(Value::StringValue(label)) = &instruction.operands[0].value {
                            self.state.program_counter = self.find_instruction_point_for_label(label) - 1;
                        } else {
                            // TODO: Error.
                        }
                    }
                } else {
                    // TODO: Error.
                }
            }
            OpCode::Pop => {
                self.state.stack.pop();
            }
            OpCode::CallFunc => {
                // Call a function, whose parameters are expected to
                // be on the stack. Pushes the function's return value,
                // if it returns one.
                if let Some(Value::StringValue(func_name)) = &instruction.operands[0].value {
                    if let Some(function) = self.library.get(func_name) {
                        let mut expected_param_count = function.param_count;
                        let actual_param_count = self.state.stack.pop().unwrap().as_number() as i8;

                        // If a function indicates -1 parameters, it takes as
                        // many parameters as it was given (i.e. it's a
                        // variadic function)
                        if expected_param_count == -1 {
                            expected_param_count = actual_param_count;
                        }

                        if expected_param_count != actual_param_count {
                            panic!(
                                "Function {} expected {}, but received {}",
                                func_name,
                                expected_param_count,
                                actual_param_count,
                            );
                        }

                        let result = if actual_param_count == 0 {
                            function.func.call(&[])
                        } else {
                            // Get the parameters, which were pushed in reverse
                            let mut parameters = vec![YarnValue::Null; actual_param_count as usize];
                            for i in (0..actual_param_count as usize).rev() {
                                let value = self.state.stack.pop().unwrap();
                                parameters[i] = value;
                            }

                            function.func.call(&parameters)
                        };

                        if let Some(result) = result {
                            // If the function returns a value, push it.
                            self.state.stack.push(result);
                        }
                    } else {
                        // TODO: Error.
                    }
                } else {
                    // TODO: Error.
                }
            }
            OpCode::PushVariable => {
                if let Some(Value::StringValue(var_name)) = &instruction.operands[0].value {
                    if let Some(val) = self.variable_storage.get(var_name) {
                        self.state.stack.push(val.clone());
                    } else {
                        // Value is undefined, so push null.
                        self.state.stack.push(YarnValue::Null);
                    }
                } else {
                    // TODO: Error.
                }
            }
            OpCode::StoreVariable => {
                if let Some(Value::StringValue(var_name)) = &instruction.operands[0].value {
                    if let Some(val) = self.state.stack.last() {
                        self.variable_storage.insert(var_name.clone(), val.clone());
                    } else {
                        // TODO: Error.
                    }
                } else {
                    // TODO: Error.
                }
            }
            OpCode::Stop => {
                self.node_complete_handler(&self.state.current_node_name);
                self.dialogue_complete_handler();
                self.execution_state = ExecutionState::Stopped;
                self.state = VmState::new();
            }
            OpCode::RunNode => {
                if let Some(YarnValue::Str(node_name)) = self.state.stack.pop() {
                    let pause = self.node_complete_handler(&self.state.current_node_name);

                    self.set_node(&node_name);

                    // Decrement program counter here, because it will
                    // be incremented when this function returns, and
                    // would mean skipping the first instruction
                    self.state.program_counter -= 1;

                    if pause {
                        self.execution_state = ExecutionState::Suspended;
                    }
                } else {
                    // TODO: Error!
                }
            }
        }
    }

    fn find_instruction_point_for_label(&self, label: &str) -> isize {
        let instruction_point = self.program.nodes.get(&self.state.current_node_name)
            .and_then(|node| node.labels.get(label));
        if let Some(&instruction_point) = instruction_point {
            instruction_point as isize
        } else {
            panic!("Unknown label {} in node {}", label, self.state.current_node_name);
        }
    }

    fn line_handler(&self, line: Line) -> bool {
        let text = self.string_table.iter()
            .find(|record| record.id == line.id)
            .map(|record| &record.text);
        if let Some(text) = text {
            println!("{}", text);
        } else {
            // TODO: Could not find line, handle error.
        }

        false
    }

    fn options_handler(&self, options: Vec<YarnOption>) {
        println!("== Choose option ==");
        for (i, opt) in options.iter().enumerate() {
            let text = self.string_table.iter()
                .find(|record| record.id == opt.line.id)
                .map(|record| &record.text);
            if let Some(text) = text {
                println!("{}: {}", i, text);
            } else {
                // TODO: Could not find line, handle error.
            }
        }
    }

    fn command_handler(&self, command: &str) -> bool {
        println!("== Command: {} ==", command);
        false
    }

    fn node_start_handler(&self, node_name: &str) -> bool {
        println!("== Starting node: {} ==", node_name);
        false
    }

    fn node_complete_handler(&self, node_name: &str) -> bool {
        println!("== Completed node: {} ==", node_name);
        false
    }

    fn dialogue_complete_handler(&self) {
        println!("== Dialogue complete ==");
    }
}
