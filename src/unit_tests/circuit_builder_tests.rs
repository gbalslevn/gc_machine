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