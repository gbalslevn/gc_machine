use std::collections::VecDeque;

use num_bigint::{ToBigUint};

use crate::{circuit_builder::CircuitBuilder, garbler::Garbler, gates::{gate_gen::GateGen, original_gate_gen::OriginalGateGen}, wires::{original_wire_gen::OriginalWireGen, wire_gen::WireGen}};

#[should_panic = "Garbler and evaluator input length must be equal"]
#[test]
fn garbler_and_evaluator_length_must_be_equal() {
     let wire_gen = OriginalWireGen::new();
    let gate_gen = OriginalGateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen); // It should only take gategen as input
    let mut circuit_builder = CircuitBuilder::new();
    let cb = circuit_builder.get_circuit_build();

    let mut garbler_input = garbler.create_circuit_input(&1.to_biguint().unwrap(), 2);
    garbler.create_circuit(&cb, &mut garbler_input, VecDeque::new());
}