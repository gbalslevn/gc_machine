use num_bigint::{ToBigUint};

use crate::{circuit_builder::CircuitBuilder};

// constant has a build id of 0 and 1
#[test]
fn const_has_specific_id() {
    let mut circuit_builder = CircuitBuilder::new();
    let cb = circuit_builder.get_circuit_build();
    let false_constant_id = cb.get_false_constant().wire_id();
    let true_constant_id = cb.get_true_constant().wire_id();
    assert!(true_constant_id == &0.to_biguint().unwrap());
    assert!(false_constant_id == &1.to_biguint().unwrap())
}

#[test]
fn gates_are_sorted_by_increasing_output_layer() {
    let mut circuit_builder = CircuitBuilder::new();
    let input_wires = circuit_builder.build_input_wires(2);
    circuit_builder.build_or(&input_wires[0], &input_wires[1]);
    let cb = circuit_builder.get_circuit_build();
    let gates = cb.get_gates();
    let mut current_output_layer = &1.to_biguint().unwrap();
    
    for gate in gates {
        assert!(gate.wo().output_layer() >= &current_output_layer);
        current_output_layer = gate.wo().output_layer()
    }
}