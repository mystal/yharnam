use std::error::Error;
use std::fs;
use std::path::PathBuf;

use prost::Message;

use yharnam::*;

mod test_plan;

fn set_up_vm(yarnc_path: &str) -> VirtualMachine {
    let _ = pretty_env_logger::try_init();

    let proto_path = PathBuf::from(yarnc_path);

    // Read the file's bytes and load a Program.
    let proto_data = fs::read(&proto_path).unwrap();
    let program = Program::decode(&*proto_data).unwrap();

    // Load LineInfos from a csv file.
    let mut csv_path = proto_path;
    csv_path.set_file_name(format!(
        "{}-Lines.csv",
        csv_path.file_stem().unwrap().to_str().unwrap()
    ));

    let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
    let _string_table: Vec<LineInfo> = csv_reader
        .deserialize()
        .map(|result| result.unwrap())
        .collect();

    let mut vm = VirtualMachine::new(program);
    vm.library.insert(
        "assert".to_string(),
        FunctionInfo::new(1, &|parameters: &[YarnValue]| {
            if !parameters[0].as_bool() {
                assert!(false, "Assertion failed");
            }
        }),
    );
    vm.library.insert(
        "add_three_operands".to_string(),
        FunctionInfo::new_returning(3, &|parameters: &[YarnValue]| {
            let res = parameters[0].add(&parameters[1]).unwrap();
            res.add(&parameters[2]).unwrap()
        }),
    );
    vm.library.insert(
        "last_value".to_string(),
        FunctionInfo::new_returning(-1, &|parameters: &[YarnValue]| {
            parameters.last().unwrap().clone()
        }),
    );

    vm
}

#[test]
fn test_commands() -> Result<(), Box<dyn Error>> {
    let mut runner = test_plan::PlanRunner::new("test_files/Commands.yarn");
    runner.run()
}

#[test]
fn test_expressions() -> Result<(), Box<dyn Error>> {
    let mut vm = set_up_vm("test_files/Expressions.yarnc");

    vm.set_node("Start")?;
    while vm.execution_state != ExecutionState::Stopped {
        vm.continue_dialogue()?;
    }

    Ok(())
}

#[test]
fn test_format_functions() -> Result<(), Box<dyn Error>> {
    let mut runner = test_plan::PlanRunner::new("test_files/FormatFunctions.yarn");
    runner.run()
}

#[test]
fn test_functions() -> Result<(), Box<dyn Error>> {
    let mut vm = set_up_vm("test_files/Functions.yarnc");

    vm.set_node("Start")?;
    while vm.execution_state != ExecutionState::Stopped {
        vm.continue_dialogue()?;
    }

    Ok(())
}

#[test]
fn test_if_statements() -> Result<(), Box<dyn Error>> {
    let mut runner = test_plan::PlanRunner::new("test_files/IfStatements.yarn");
    runner.run()
}

#[test]
fn test_inline_expressions() -> Result<(), Box<dyn Error>> {
    let mut runner = test_plan::PlanRunner::new("test_files/InlineExpressions.yarn");
    runner.run()
}

#[test]
fn test_shortcut_options() -> Result<(), Box<dyn Error>> {
    let mut runner = test_plan::PlanRunner::new("test_files/ShortcutOptions.yarn");
    runner.run()
}

#[test]
fn test_smileys() -> Result<(), Box<dyn Error>> {
    let mut runner = test_plan::PlanRunner::new("test_files/Smileys.yarn");
    runner.run()
}

#[test]
fn test_tags() -> Result<(), Box<dyn Error>> {
    let mut runner = test_plan::PlanRunner::new("test_files/Tags.yarn");
    runner.run()
}

#[test]
fn test_types() -> Result<(), Box<dyn Error>> {
    let mut vm = set_up_vm("test_files/Types.yarnc");

    vm.set_node("Start")?;
    while vm.execution_state != ExecutionState::Stopped {
        vm.continue_dialogue()?;
    }

    Ok(())
}

#[test]
fn test_variable_storage() -> Result<(), Box<dyn Error>> {
    let mut vm = set_up_vm("test_files/VariableStorage.yarnc");

    vm.set_node("Start")?;
    while vm.execution_state != ExecutionState::Stopped {
        vm.continue_dialogue()?;
    }

    Ok(())
}
