use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

use prost::Message;

use yharnam::*;

fn set_up_vm(yarnc_path: &str) -> VirtualMachine {
    let _ = pretty_env_logger::try_init();

    let proto_path = PathBuf::from(yarnc_path);

    // Read the file's bytes and load a Program.
    let proto_data = fs::read(&proto_path)
        .unwrap();
    let program = Program::decode(&mut Cursor::new(&proto_data))
        .unwrap();

    // Load Records from a csv file.
    let mut csv_path = proto_path;
    csv_path.set_extension("csv");
    let mut csv_reader = csv::Reader::from_path(csv_path)
        .unwrap();
    let string_table: Vec<Record> = csv_reader.deserialize()
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
fn test_expressions() {
    let mut vm = set_up_vm("test_files/Expressions.yarn.yarnc");

    vm.set_node("Start");
    while vm.execution_state != ExecutionState::Stopped {
        vm.continue_dialogue();
    }
}

#[test]
fn test_functions() {
    let mut vm = set_up_vm("test_files/Functions.yarn.yarnc");

    vm.set_node("Start");
    while vm.execution_state != ExecutionState::Stopped {
        vm.continue_dialogue();
    }
}

#[test]
fn test_types() {
    let mut vm = set_up_vm("test_files/Types.yarn.yarnc");

    vm.set_node("Start");
    while vm.execution_state != ExecutionState::Stopped {
        vm.continue_dialogue();
    }
}

#[test]
fn test_variable_storage() {
    let mut vm = set_up_vm("test_files/VariableStorage.yarn.yarnc");

    vm.set_node("Start");
    while vm.execution_state != ExecutionState::Stopped {
        vm.continue_dialogue();
    }
}
