//! Contains functions that can be inserted into the library

use std::collections::HashMap;

use crate::{FunctionInfo, VirtualMachine, YarnValue};

/// Adds mathemtical functions such as addition, subtraction and so on to the library.
pub fn add_mathetmatical_functions(library: &mut HashMap<String, FunctionInfo>) {
    library.insert(
        "Add".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            parameters[0].add(&parameters[1]).unwrap()
        }),
    );

    library.insert(
        "Minus".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            parameters[0].sub(&parameters[1]).unwrap()
        }),
    );

    library.insert(
        "UnaryMinus".to_string(),
        FunctionInfo::new_returning(1, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            parameters[0].neg()
        }),
    );

    library.insert(
        "Divide".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            parameters[0].div(&parameters[1]).unwrap()
        }),
    );

    library.insert(
        "Multiply".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            parameters[0].mul(&parameters[1]).unwrap()
        }),
    );

    library.insert(
        "Modulo".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            parameters[0].rem(&parameters[1]).unwrap()
        }),
    );

    library.insert(
        "EqualTo".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0] == parameters[1]).into()
        }),
    );

    library.insert(
        "NotEqualTo".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0] != parameters[1]).into()
        }),
    );

    library.insert(
        "GreaterThan".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0] > parameters[1]).into()
        }),
    );

    library.insert(
        "GreaterThanOrEqualTo".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0] >= parameters[1]).into()
        }),
    );

    library.insert(
        "LessThan".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0] < parameters[1]).into()
        }),
    );

    library.insert(
        "LessThanOrEqualTo".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0] <= parameters[1]).into()
        }),
    );
}

/// Adds logic functions to the library, such as "and", "or" and so on.
pub fn add_logic_functions(library: &mut HashMap<String, FunctionInfo>) {
    library.insert(
        "And".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0].as_bool() && parameters[1].as_bool()).into()
        }),
    );

    library.insert(
        "Or".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0].as_bool() || parameters[1].as_bool()).into()
        }),
    );

    library.insert(
        "Xor".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0].as_bool() ^ parameters[1].as_bool()).into()
        }),
    );

    library.insert(
        "Not".to_string(),
        FunctionInfo::new_returning(1, &|_vm: &VirtualMachine, parameters: &[YarnValue]| {
            (!parameters[0].as_bool()).into()
        }),
    );
}

/// Adds functions such as "visited" and "visited_count" to the library
pub fn add_visited_functions(library: &mut HashMap<String, FunctionInfo>) {
    library.insert(
        "visited".to_string(),
        FunctionInfo::new_returning(1, &|vm: &VirtualMachine, parameters: &[YarnValue]| {
            (*vm.visit_counter
                .get(&parameters[0].as_string())
                .unwrap_or_else(|| &0)
                > 0)
            .into()
        }),
    );

    library.insert(
        "visited_count".to_string(),
        FunctionInfo::new_returning(1, &|vm: &VirtualMachine, parameters: &[YarnValue]| {
            YarnValue::Number(
                *vm.visit_counter
                    .get(&parameters[0].as_string())
                    .unwrap_or_else(|| &0) as f32,
            )
        }),
    );
}

