//! Contains functions that can be inserted into the library

use std::collections::HashMap;

use crate::{FunctionInfo, VirtualMachine, YarnValue};

/// Adds mathematical functions such as addition, subtraction and so on to the library.
pub fn add_mathematical_functions(library: &mut HashMap<String, FunctionInfo>) {
    library.insert(
        "Add".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            parameters[0].add(&parameters[1]).unwrap()
        }),
    );

    library.insert(
        "Minus".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            parameters[0].sub(&parameters[1]).unwrap()
        }),
    );

    library.insert(
        "UnaryMinus".to_string(),
        FunctionInfo::new_returning(1, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            parameters[0].neg()
        }),
    );

    library.insert(
        "Divide".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            parameters[0].div(&parameters[1]).unwrap()
        }),
    );

    library.insert(
        "Multiply".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            parameters[0].mul(&parameters[1]).unwrap()
        }),
    );

    library.insert(
        "Modulo".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            parameters[0].rem(&parameters[1]).unwrap()
        }),
    );

    library.insert(
        "EqualTo".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0] == parameters[1]).into()
        }),
    );

    library.insert(
        "NotEqualTo".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0] != parameters[1]).into()
        }),
    );

    library.insert(
        "GreaterThan".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0] > parameters[1]).into()
        }),
    );

    library.insert(
        "GreaterThanOrEqualTo".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0] >= parameters[1]).into()
        }),
    );

    library.insert(
        "LessThan".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0] < parameters[1]).into()
        }),
    );

    library.insert(
        "LessThanOrEqualTo".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0] <= parameters[1]).into()
        }),
    );
}

/// Adds logic functions to the library, such as "and", "or" and so on.
pub fn add_logic_functions(library: &mut HashMap<String, FunctionInfo>) {
    library.insert(
        "And".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0].as_bool() && parameters[1].as_bool()).into()
        }),
    );

    library.insert(
        "Or".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0].as_bool() || parameters[1].as_bool()).into()
        }),
    );

    library.insert(
        "Xor".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            (parameters[0].as_bool() ^ parameters[1].as_bool()).into()
        }),
    );

    library.insert(
        "Not".to_string(),
        FunctionInfo::new_returning(1, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            (!parameters[0].as_bool()).into()
        }),
    );
}

/// Adds functions such as "visited" and "visited_count" to the library
pub fn add_visited_functions(library: &mut HashMap<String, FunctionInfo>) {
    library.insert(
        "visited".to_string(),
        FunctionInfo::new_returning(1, &|vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            (*vm.visit_counter
                .get(&parameters[0].as_string())
                .unwrap_or_else(|| &0)
                > 0)
            .into()
        }),
    );

    library.insert(
        "visited_count".to_string(),
        FunctionInfo::new_returning(1, &|vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            YarnValue::Number(
                *vm.visit_counter
                    .get(&parameters[0].as_string())
                    .unwrap_or_else(|| &0) as f32,
            )
        }),
    );
}

/// Adds function
pub fn add_number_utility_functions(library: &mut HashMap<String, FunctionInfo>) {
    library.insert(
        "floor".to_string(),
        FunctionInfo::new_returning(1, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            let number = parameters[0].as_number();
            YarnValue::Number(number.floor())
        }),
    );

    library.insert(
        "ceil".to_string(),
        FunctionInfo::new_returning(1, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            let number = parameters[0].as_number();
            YarnValue::Number(number.ceil())
        }),
    );

    library.insert(
        "decimal".to_string(),
        FunctionInfo::new_returning(1, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            let number = parameters[0].as_number();
            YarnValue::Number(number.fract().abs())
        }),
    );

    library.insert(
        "dec".to_string(),
        FunctionInfo::new_returning(1, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            let number = parameters[0].as_number();
            YarnValue::Number(if number.floor() == number {
                number - 1.
            } else {
                number.floor()
            })
        }),
    );

    library.insert(
        "inc".to_string(),
        FunctionInfo::new_returning(1, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            let number = parameters[0].as_number();
            YarnValue::Number(if number.ceil() == number {
                number + 1.
            } else {
                number.ceil()
            })
        }),
    );

    library.insert(
        "round".to_string(),
        FunctionInfo::new_returning(1, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            let number = parameters[0].as_number();
            YarnValue::Number(number.round())
        }),
    );

    library.insert(
        "round_places".to_string(),
        FunctionInfo::new_returning(2, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            let number = parameters[0].as_number();
            let num_places = parameters[1].as_number() as u32;
            let multiplier = 10u32.pow(num_places) as f32;

            YarnValue::Number((number * multiplier).round() / multiplier)
        }),
    );
}

/// Adds random number functions
#[cfg(feature = "random")]
pub fn add_random_functions(library: &mut HashMap<String, FunctionInfo>) {
    use rand::Rng;

    library.insert(
        "dice".to_string(),
        FunctionInfo::new_returning(1, &|vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            YarnValue::Number(vm.rand.gen_range(1..=(parameters[0].as_number() as u32)) as f32)
        }),
    );

    library.insert(
        "random".to_string(),
        FunctionInfo::new_returning(0, &|vm: &mut VirtualMachine, _parameters: &[YarnValue]| {
            YarnValue::Number(vm.rand.gen::<f32>())
        }),
    );

    library.insert(
        "random_range".to_string(),
        FunctionInfo::new_returning(2, &|vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            let a = parameters[0].as_number() as u32;
            let b = parameters[1].as_number() as u32;
            YarnValue::Number(vm.rand.gen_range(a..=b) as f32)
        }),
    );

    library.insert(
        "random_test".to_string(),
        FunctionInfo::new_returning(1, &|vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            let threshold = parameters[0].as_number() as f64;
            YarnValue::Bool(vm.rand.gen_bool(threshold))
        }),
    );
}
