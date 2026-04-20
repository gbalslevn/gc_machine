use gc_machine::circuit_builder::{CircuitBuild, CircuitBuilder};
use circuit_macro::{circuit_fn, circuit};

#[circuit_fn]
fn add(garbler_input: usize, evaluator_input: usize) -> usize {
    garbler_input + evaluator_input   
}

#[test]
fn can_produce_adder() {
    let cb: CircuitBuild = circuit! { add() };

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
    let cb: CircuitBuild = circuit! { multiplication() };

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
    let cb: CircuitBuild = circuit! { is_equal() };

    // Manual equivalent for assertion
    let mut manual = CircuitBuilder::new();
    let (g, e) = manual.set_input_wires(1);
    manual.build_is_equal(&g, &e);

    assert_eq!(cb, manual.get_circuit_build());
}