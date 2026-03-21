use std::collections::{HashMap, HashSet};
use num_bigint::{BigUint};
use crate::circuit_builder::{CircuitBuild, CircuitBuilder, GateBuild};

#[test]
fn gates_are_sorted_by_increasing_output_layer() {
    let mut circuit_builder = CircuitBuilder::new();
    let input_wires = circuit_builder.build_input_wires(2);
    circuit_builder.build_is_equal(input_wires);
    let cb = circuit_builder.get_circuit_build();
    let gates = cb.get_gates();
    let mut current_output_layer = &1;
    
    for gate in gates {
        assert!(gate.wo().ready_at_layer() >= &current_output_layer);
        current_output_layer = gate.wo().ready_at_layer()
    }
}

#[test]
fn two_ifs_creates_3_branches() {
    let (cb, _input_gates) = get_nested_if_build(); 
    let gates = cb.get_gates();

    // We expect all branches to be present for the last gate as the output could have come from all branches
    assert_eq!(gates[gates.len() - 1].branches().len(), 3);
}
#[test]
fn branches_assigned_correctly_for_two_ifs() {
    let (cb, input_gates) = get_nested_if_build(); 
    let gates = cb.get_gates();
    
    // Input gates has a single branch
    for gate in input_gates {
        assert_eq!(gate.branches().len(), 1)
    }

     // A gate should not have dublicate branches
    for gate in gates {
        let mut used_branches = HashSet::new();    
        for branch in gate.branches() {
            assert!(used_branches.insert(branch), "Dublicate branch found for gate_id {} in branches {:?}", gate.wo().wire_id(), gate.branches())
        }
    }
}

fn get_nested_if_build() -> (CircuitBuild, Vec<GateBuild>) {
    let mut builder = CircuitBuilder::new();
    
    let inputs = builder.build_input_wires(4); 
    let cond = &inputs[0];
    let wi = &inputs[1];
    let wj = &inputs[2];
    let wz = &inputs[3];

    // First if: 
    let and_0 = builder.build_and_output(wi, wi);
    let and_1 = builder.build_and_output(wj, wj);
    let if_out = builder.build_if(cond, &and_0, &and_1);

    // Second if, nested
    let and_2 = builder.build_and_output(&wz, &wz);
    builder.build_if(cond, &if_out, &and_2);

    let cb = builder.get_circuit_build();
    let gates = cb.get_gates();

    let id_to_gate_index: HashMap<BigUint, usize> = gates.iter().enumerate().map(|(idx, gate)| (gate.wo().wire_id().clone(), idx)).collect();
    let and_0_build = &gates[id_to_gate_index.get(and_0.wire_id()).unwrap().clone()];
    let and_1_build = &gates[id_to_gate_index.get(and_1.wire_id()).unwrap().clone()];
    let and_2_build = &gates[id_to_gate_index.get(and_2.wire_id()).unwrap().clone()];

    (cb.clone(), vec![and_0_build.clone(), and_1_build.clone(), and_2_build.clone()])
}