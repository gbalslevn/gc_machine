use num_bigint::{ToBigUint};

use crate::{circuit_builder::CircuitBuilder};

#[test] 
fn wire_id_increments_by_amount_of_outputs_created() {
    let mut circuit_builder = CircuitBuilder::new();
    let cb = circuit_builder.get_circuit_build();
    let gates_at_the_start = cb.get_gates();
    assert!(gates_at_the_start.len() == 0);
    
    let input_wires = circuit_builder.build_input_wires(2);
    circuit_builder.build_xnor(&input_wires[0], &input_wires[1]);

    let cb = circuit_builder.get_circuit_build();
    let gates = cb.get_gates();
    let number_of_gates = gates.len();
    let start_id = 2; // Two first outputs are constant values, true and false.

    let final_wire_id = gates[number_of_gates - 1].wo().wire_id();
    assert!(final_wire_id == &(number_of_gates - 1 + start_id).to_biguint().unwrap());   
}

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
    circuit_builder.build_is_equal(1);
    let cb = circuit_builder.get_circuit_build();
    let gates = cb.get_gates();
    let mut current_output_layer = &1.to_biguint().unwrap();
    
    for gate in gates {
        assert!(gate.wo().output_layer() >= &current_output_layer);
        current_output_layer = gate.wo().output_layer()
    }
}