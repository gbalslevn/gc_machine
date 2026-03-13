use num_bigint::{ToBigUint};

use crate::{circuit_builder::CircuitBuilder};

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