// #![warn(missing_docs)]

use std::{collections::HashMap, convert::TryInto};

use errors::VmError;
use log::*;
use serde::Deserialize;

pub use crate::{utils::*, value::YarnValue, yarn_proto::Program};

pub mod yarn_proto {
    include!(concat!(env!("OUT_DIR"), "/yarn.rs"));
}

pub mod errors;
mod utils;
mod value;

#[derive(Debug, Deserialize)]
pub struct LineInfo {
    pub id: String,
    pub text: String,
    pub file: String,
    pub node: String,
    #[serde(rename = "lineNumber")]
    pub line_number: u32,
}

/// A line of dialogue, sent from the [`VirtualMachine`] to the game.
///
/// When the game receives a `Line`, it should do the following things to prepare the line for
/// presentation to the user.
///
/// 1. Use the value in the `id` field to look up the appropriate user-facing text in the string
/// table.
///
/// 2. For each of the entries in the `substitutions` field, replace the corresponding placeholder
/// with the entry. That is, the text "`{0}`" should be replaced with the value of
/// `substitutions[0]`, "`{1}`" with `substitutions[1]`, and so on.
///
/// 3. Use [`expand_format_functions`] to expand all [format functions](
/// https://yarnspinner.dev/docs/syntax#format-functions) in the line.
///
/// You do not create instances of this struct yourself. They are created by the [`VirtualMachine`]
/// during program execution.
#[derive(Debug, Clone)]
pub struct Line {
    pub id: String,
    pub substitutions: Vec<String>,
}

impl Line {
    fn new(id: String, substitutions: Vec<String>) -> Self {
        Self { id, substitutions }
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

pub type ReturningFunction = dyn Fn(&[YarnValue]) -> YarnValue + Send + Sync;
pub type Function = dyn Fn(&[YarnValue]) + Send + Sync;

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

enum ParamCount {
    N(u8),
    Variadic,
}

impl From<i8> for ParamCount {
    fn from(val: i8) -> Self {
        if val >= 0 {
            Self::N(val as u8)
        } else {
            Self::Variadic
        }
    }
}

pub struct FunctionInfo {
    param_count: ParamCount,
    func: YarnFunction,
}

impl FunctionInfo {
    pub fn new(param_count: i8, func: &'static Function) -> Self {
        Self {
            param_count: param_count.into(),
            func: YarnFunction::Void(func),
        }
    }

    pub fn new_returning(param_count: i8, func: &'static ReturningFunction) -> Self {
        Self {
            param_count: param_count.into(),
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

pub enum SuspendReason {
    Line(Line),
    Options(Vec<YarnOption>),
    Command(String),
    NodeChange { start: String, end: String },
    DialogueComplete(String),
    InvalidOption(String),
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
}

impl VirtualMachine {
    pub fn new(program: Program) -> Self {
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
            FunctionInfo::new_returning(1, &|parameters: &[YarnValue]| parameters[0].neg()),
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
        }
    }

    pub fn set_node(&mut self, node_name: &str) -> Result<(), VmError> {
        if self.program.nodes.is_empty() {
            return Err("No nodes available in the program".into());
        }

        if !self.program.nodes.contains_key(node_name) {
            self.execution_state = ExecutionState::Stopped;
            return Err(format!("Program does not contain node {node_name}").into());
        }

        self.state = VmState::new();
        self.state.current_node_name = node_name.to_string();

        // TODO: Suspending makes sense to me, but is it correct?
        self.execution_state = ExecutionState::Suspended;

        Ok(())
    }

    // TODO: Return the reason why we stopped execution.
    // Either Line, Options, Command, NodeStart?, NodeEnd?, DialogeEnd
    pub fn continue_dialogue(&mut self) -> Result<SuspendReason, VmError> {
        if self.state.current_node_name.is_empty() {
            return Err("Cannot continue running dialogue. No node has been selected.".into());
        }

        if self.execution_state == ExecutionState::WaitingOnOptionSelection {
            return Err("Unable to continue dialogue, waiting on option selection".into());
        }

        self.execution_state = ExecutionState::Running;

        // Execute instructions until something forces us to stop
        loop {
            let instruction_count = if !self.state.current_node_name.is_empty() {
                self.program.nodes[&self.state.current_node_name]
                    .instructions
                    .len()
            } else {
                // No node is running, so return 0.
                0
            };

            // If we've reached the end of a node, stop execution.
            if self.state.program_counter as usize >= instruction_count {
                let last_node = self.state.current_node_name.clone();
                self.execution_state = ExecutionState::Stopped;
                self.state = VmState::new();
                // dialogue.LogDebugMessage ("Run complete.");
                return Ok(SuspendReason::DialogueComplete(last_node));
            }

            let current_instruction = {
                let current_node = &self.program.nodes[&self.state.current_node_name];
                current_node.instructions[self.state.program_counter as usize].clone()
            };

            let suspend = self.run_instruction(current_instruction);

            self.state.program_counter += 1;

            match suspend {
                Ok(suspend) => {
                    // if we have a suspension reason, break out of the loop
                    return Ok(suspend);
                }
                Err(VmError::NoOperation) => {
                    // If there is a no-op, just continue
                }
                Err(e) => {
                    // But raise all other errors
                    return Err(e);
                }
            }
        }
    }

    pub fn set_selected_option(&mut self, selected_option_id: u32) -> Result<(), VmError> {
        let selected_option_id = selected_option_id as usize;

        if self.execution_state != ExecutionState::WaitingOnOptionSelection {
            return Err("set_selected_option was called, but Dialogue wasn't waiting for a selection. This method should only be called after the Dialogue is waiting for the user to select an option.".into());
        }

        if selected_option_id >= self.state.current_options.len() {
            return Err(format!("{selected_option_id} is not a valid option ID (expected a number between 0 and {}.", self.state.current_options.len() - 1).into());
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

        Ok(())
    }

    fn run_instruction(
        &mut self,
        instruction: yarn_proto::Instruction,
    ) -> Result<SuspendReason, VmError> {
        use yarn_proto::{instruction::OpCode, operand::Value};

        let opcode = match instruction.opcode.try_into() {
            Ok(opcode) => opcode,
            Err(err) => {
                return Err(
                    format!("Error decoding opcode {}: {:?}", instruction.opcode, err).into(),
                )
            }
        };

        debug!("Running {:?} {:?}", opcode, instruction.operands);

        match opcode {
            OpCode::JumpTo => {
                if let Some(Value::StringValue(label)) = &instruction.operands[0].value {
                    self.state.program_counter = self.find_instruction_point_for_label(label) - 1;
                } else {
                    return Err("Invalid jump to - no label in the value".into());
                }
            }
            OpCode::Jump => {
                if let Some(YarnValue::Str(label)) = self.state.stack.last() {
                    self.state.program_counter = self.find_instruction_point_for_label(label) - 1;
                } else {
                    return Err("Invalid jump - found no items in the stack".into());
                }
            }
            OpCode::RunLine => {
                // Looks up a string from the string table and passes it to the client as a line.
                if let Some(Value::StringValue(string_key)) = &instruction.operands[0].value {
                    let mut substitutions = Vec::new();

                    // The second operand, if provided (compilers prior
                    // to v1.1 don't include it), indicates the number
                    // of expressions in the command. We need to pop
                    // these values off the stack and deliver them to
                    // the line handler.
                    if let Some(Value::FloatValue(expression_count)) =
                        instruction.operands.get(1).and_then(|o| o.value.as_ref())
                    {
                        let expression_count = *expression_count as u32;
                        substitutions.resize(expression_count as usize, String::new());

                        for expression_index in (0..expression_count as usize).rev() {
                            let substitution = self.state.stack.pop().unwrap().as_string();
                            // TODO: Avoid bounds check due to indexing.
                            substitutions[expression_index] = substitution;
                        }
                    }

                    self.execution_state = ExecutionState::Suspended;
                    let line = Line::new(string_key.clone(), substitutions);
                    return Ok(SuspendReason::Line(line));
                } else {
                    return Err("Invalid run line - no label in the value".into());
                }
            }
            OpCode::RunCommand => {
                // Passes a string to the client as a custom command
                if let Some(Value::StringValue(command)) = &instruction.operands[0].value {
                    let mut command_text = command.clone();

                    // The second operand, if provided (compilers prior
                    // to v1.1 don't include it), indicates the number
                    // of expressions in the command. We need to pop
                    // these values off the stack and deliver them to
                    // the line handler.
                    if let Some(Value::FloatValue(expression_count)) =
                        instruction.operands.get(1).and_then(|o| o.value.as_ref())
                    {
                        let expression_count = *expression_count as u32;

                        // Get the values from the stack, and
                        // substitute them into the command text
                        for expression_index in (0..expression_count).rev() {
                            let substitution = self.state.stack.pop().unwrap().as_string();

                            // TODO: Try using String::replace_range.
                            command_text = command_text.replacen(
                                &format!("{{{}}}", expression_index),
                                &substitution,
                                1,
                            );
                        }
                    }

                    self.execution_state = ExecutionState::Suspended;
                    return Ok(SuspendReason::Command(command_text));
                } else {
                    return Err("Invalid run command - no label in the value".into());
                }
            }
            OpCode::AddOption => {
                let line = if let Some(Value::StringValue(opt)) =
                    instruction.operands.get(0).and_then(|o| o.value.as_ref())
                {
                    let mut substitutions = Vec::new();

                    // get the number of expressions that we're
                    // working with out of the third operand
                    if let Some(Value::FloatValue(expression_count)) =
                        instruction.operands.get(2).and_then(|o| o.value.as_ref())
                    {
                        let expression_count = *expression_count as u32;
                        substitutions.resize(expression_count as usize, String::new());

                        // pop the expression values off the stack in
                        // reverse order, and store the list of substitutions
                        for expression_index in (0..expression_count as usize).rev() {
                            let substitution = self.state.stack.pop().unwrap().as_string();
                            // TODO: Avoid bounds check due to indexing.
                            substitutions[expression_index] = substitution;
                        }
                    }
                    Line::new(opt.clone(), substitutions)
                } else {
                    return Err(
                        "Invalid add option - unable to find instruction operand at index 0".into(),
                    );
                };

                let node_name = if let Some(Value::StringValue(opt)) =
                    instruction.operands.get(1).and_then(|o| o.value.as_ref())
                {
                    opt.clone()
                } else {
                    return Err(
                        "Invalid add option - unable to find instruction operand at index 1".into(),
                    );
                };

                self.state.current_options.push((line, node_name));
            }
            OpCode::ShowOptions => {
                // If we have no options to show, immediately stop.
                if self.state.current_options.is_empty() {
                    self.execution_state = ExecutionState::Stopped;
                    let last_node = self.state.current_node_name.clone();
                    self.state = VmState::new();
                    return Ok(SuspendReason::DialogueComplete(last_node));
                }

                // Present the list of options to the user and let them pick
                let mut options = Vec::new();

                for (i, opt) in self.state.current_options.iter().enumerate() {
                    options.push(YarnOption::new(opt.0.clone(), i as u32, opt.1.clone()));
                }

                // We can't continue until our client tell us which option to pick.
                self.execution_state = ExecutionState::WaitingOnOptionSelection;

                return Ok(SuspendReason::Options(options));
            }
            OpCode::PushString => {
                if let Some(Value::StringValue(val)) = &instruction.operands[0].value {
                    self.state.stack.push(YarnValue::Str(val.clone()));
                } else {
                    return Err("Invalid push string - bad operand".into());
                }
            }
            OpCode::PushFloat => {
                if let Some(Value::FloatValue(val)) = &instruction.operands[0].value {
                    self.state.stack.push(YarnValue::Number(*val));
                } else {
                    return Err("Invalid push float - bad operand".into());
                }
            }
            OpCode::PushBool => {
                if let Some(Value::BoolValue(val)) = &instruction.operands[0].value {
                    self.state.stack.push(YarnValue::Bool(*val));
                } else {
                    return Err("Invalid push bool - bad operand".into());
                }
            }
            OpCode::PushNull => {
                self.state.stack.push(YarnValue::Null);
            }
            OpCode::JumpIfFalse => {
                // Jump to a named label if the value on the top of the stack
                // evaluates to the boolean value 'false'.
                if let Some(val) = self.state.stack.last() {
                    if !val.as_bool() {
                        if let Some(Value::StringValue(label)) = &instruction.operands[0].value {
                            self.state.program_counter =
                                self.find_instruction_point_for_label(label) - 1;
                        } else {
                            return Err("Invalid jump if false - operand 0 has no value".into());
                        }
                    }
                } else {
                    return Err("Invalid jump if false - no items in the stack".into());
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
                        let actual_param_count = self.state.stack.pop().unwrap().as_number() as u8;

                        // If a function is variadic, it takes as many parameters as it was given.
                        let expected_param_count = match function.param_count {
                            ParamCount::N(n) => n,
                            ParamCount::Variadic => actual_param_count,
                        };

                        if expected_param_count != actual_param_count {
                            // panic is ok here as this should be a "compile time" error.
                            panic!(
                                "Function {} expected {}, but received {}",
                                func_name, expected_param_count, actual_param_count,
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
                        return Err(VmError::MissingLibraryFunction(func_name.clone()));
                    }
                } else {
                    return Err("Invalid call func - index 0 has no value".into());
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
                    return Err("Invalid push variable - index 0 has no value".into());
                }
            }
            OpCode::StoreVariable => {
                if let Some(Value::StringValue(var_name)) = &instruction.operands[0].value {
                    if let Some(val) = self.state.stack.last() {
                        self.variable_storage.insert(var_name.clone(), val.clone());
                    } else {
                        return Err("Invalid push variable - no items in the stack".into());
                    }
                } else {
                    return Err("Invalid push variable - index 0 has no value".into());
                }
            }
            OpCode::Stop => {
                self.execution_state = ExecutionState::Stopped;
                let last_node = self.state.current_node_name.clone();
                self.state = VmState::new();
                return Ok(SuspendReason::DialogueComplete(last_node));
            }
            OpCode::RunNode => {
                if let Some(YarnValue::Str(node_name)) = self.state.stack.pop() {
                    let old_node = self.state.current_node_name.clone();

                    self.set_node(&node_name)?;

                    // Decrement program counter here, because it will
                    // be incremented when this function returns, and
                    // would mean skipping the first instruction
                    self.state.program_counter -= 1;

                    self.execution_state = ExecutionState::Suspended;

                    return Ok(SuspendReason::NodeChange {
                        start: node_name,
                        end: old_node,
                    });
                } else {
                    return Err("Invalid run node - no items in the stack".into());
                }
            }
        }

        Err(VmError::NoOperation)
    }

    fn find_instruction_point_for_label(&self, label: &str) -> isize {
        let instruction_point = self
            .program
            .nodes
            .get(&self.state.current_node_name)
            .and_then(|node| node.labels.get(label));
        if let Some(&instruction_point) = instruction_point {
            instruction_point as isize
        } else {
            panic!(
                "Unknown label {} in node {}",
                label, self.state.current_node_name
            );
        }
    }
}
