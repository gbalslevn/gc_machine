use gc_machine::circuit_builder::{CircuitBuild, CircuitBuilder};
use circuit_macro::{circuit_fn, circuit};

#[circuit_fn]
fn add(garbler_input: usize, evaluator_input: usize) -> usize {
    garbler_input + evaluator_input   
}

#[test]
fn can_produce_adder() {
    let cb: CircuitBuild = circuit! { add };

    // Manual equivalent for assertion
    let mut manuel = CircuitBuilder::new();
    let (g, e) = manuel.set_input_wires(1);
    manuel.build_adder(&g, &e);

    assert_eq!(cb, manuel.get_circuit_build());
}

#[circuit_fn]
fn multiplication(garbler_input: usize, evaluator_input: usize) -> usize {
    garbler_input * evaluator_input   
}

#[test]
fn can_produce_multiplication() {
    let cb: CircuitBuild = circuit! { multiplication };

    // Manual equivalent for assertion
    let mut manual = CircuitBuilder::new();
    let (g, e) = manual.set_input_wires(1);
    manual.build_multiplier(&g, &e);

    assert_eq!(cb, manual.get_circuit_build());
}

#[circuit_fn]
fn is_equal(garbler_input: usize, evaluator_input: usize) -> bool {
    garbler_input == evaluator_input   
}

#[test]
fn can_produce_is_equal() {
    let cb: CircuitBuild = circuit! { is_equal };

    // Manual equivalent for assertion
    let mut manual = CircuitBuilder::new();
    let (g, e) = manual.set_input_wires(1);
    manual.build_is_equal(&g, &e);

    assert_eq!(cb, manual.get_circuit_build());
}

#[circuit_fn]
fn produce_naive_if(garbler_input: usize, evaluator_input: usize) -> usize {
    if garbler_input == evaluator_input {
        garbler_input + garbler_input
    } else {
        garbler_input
    }
}

#[test]
fn can_produce_naive_if() {
    let cb : CircuitBuild = circuit!(produce_naive_if);

    // Manual equivalent for assertion
    let mut manual = CircuitBuilder::new();
    let (g, e) = manual.set_input_wires(1);

    let is_equal = manual.build_is_equal(&g, &e);
    let true_case = manual.build_adder(&g, &g);
    manual.build_if(&is_equal, &true_case, &g);

    assert_eq!(cb, manual.get_circuit_build());
}

// #[circuit_fn]
// fn produce_stacked_if(garbler_input: usize, evaluator_input: usize) -> usize {
//     if garbler_input == evaluator_input {
//         garbler_input + garbler_input
//     } else {
//         garbler_input
//     }
// }

// #[test]
// fn can_produce_stacked_if() {
//     todo!()
// }

// #[test]
// fn can_produce_nested_stacked_if() {
//     todo!()
// }

// #[circuit_fn] 
// fn produce_constant(garbler_input: usize, evaluator_input: usize) -> usize {
//     garbler_input + 2
// }
// #[test]
// fn can_use_constants() {

// }

// #[test]
// fn can_use_constants_which_is_not_numbers() {
//     // appending a string somewhere
// }