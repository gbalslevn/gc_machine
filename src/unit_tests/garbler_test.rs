use std::collections::VecDeque;

use num_bigint::{ToBigUint};

use crate::{circuit_builder::CircuitBuilder, evaluator::{evaluator::Evaluator, original_evaluator::OriginalEvaluator}, garbler::Garbler, gates::{gate_gen::GateGen, original_gate_gen::OriginalGateGen}};

#[should_panic = "Garbler and evaluator input length must be equal"]
#[test]
fn garbler_and_evaluator_length_must_be_equal() {
    let gate_gen = OriginalGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut circuit_builder = CircuitBuilder::new();
    circuit_builder.set_input_wires(1);
    let cb = circuit_builder.get_circuit_build();

    let garbler_input = garbler.create_circuit_input(&1.to_biguint().unwrap(), 2);
    garbler.create_circuit(&cb, &garbler_input, &VecDeque::new());
}

#[should_panic = "Garblers input cannot be greater than what is set in the circuitbuild"]
#[test]
fn garbler_cannot_provide_input_larger_than_max_input_length() {
    let gate_gen = OriginalGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let evaluator = OriginalEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();
    circuit_builder.set_input_wires(1); // set max input to 1
    let cb = circuit_builder.get_circuit_build();
    
    // Proivde input of length 2
    let (evaluator_input, _) = evaluator.create_circuit_input(&1.to_biguint().unwrap(), 2);
    let garbler_input = garbler.create_circuit_input(&1.to_biguint().unwrap(), 2);

    garbler.create_circuit(&cb, &garbler_input, &evaluator_input);
}