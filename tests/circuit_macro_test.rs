use gc_machine::circuit_builder::{BuildBlock, CircuitBuild, CircuitBuilder};
use circuit_macro::{circuit_fn, circuit};
use num_bigint::{ToBigUint};

#[circuit_fn(input_bits = 1)]
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

#[circuit_fn(input_bits = 1)]
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

#[circuit_fn(input_bits = 1)]
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

#[circuit_fn(input_bits = 1, naive_stack = true)]
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

    let is_equal = manual.build_is_equal(&g, &e).output[0].clone();
    let true_case = manual.build_adder(&g, &g);
    manual.build_if(&is_equal, &true_case, &BuildBlock{output : g, builds : vec![]});

    assert_eq!(cb, manual.get_circuit_build());
}

#[circuit_fn(input_bits = 1)]
fn produce_add_variables(__: usize, ___: usize) -> usize {
    let a = 2;
    let b = 2;
    a + b
}

#[test]
fn can_produce_add_variables() {
    let cb: CircuitBuild = circuit!(produce_add_variables);

    // Manual equivalent for assertion
    let mut manual = CircuitBuilder::new();
    let (_, _) = manual.set_input_wires(1);

    let a = 2;
    let a_build = manual.build_variable(a.to_biguint().unwrap().to_bytes_le());
    let b = 2;
    let b_build = manual.build_variable(b.to_biguint().unwrap().to_bytes_le());
    manual.build_adder(&a_build.output, &b_build.output);
    let manual_cb = manual.get_circuit_build();
    assert_eq!(cb, manual_cb)
}