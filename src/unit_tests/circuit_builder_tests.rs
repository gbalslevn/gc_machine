use std::{cmp::max, collections::{HashMap, HashSet}};
use num_bigint::{BigUint, ToBigUint};
use crate::circuit_builder::{CircuitBuild, CircuitBuilder, GateBuild};

#[test]
fn gates_are_sorted_by_increasing_output_layer() {
    let mut circuit_builder = CircuitBuilder::new();
    let (input_a, input_b) = circuit_builder.set_input_wires(1);
    circuit_builder.build_is_equal(&input_a, &input_b);
    circuit_builder.set_input_wires(1);
    let cb = circuit_builder.get_circuit_build();
    let gates = cb.get_gates();
    let mut current_output_layer = &1;
    
    for gate in gates {
        assert!(gate.wo().ready_at_layer() >= &current_output_layer);
        current_output_layer = gate.wo().ready_at_layer()
    }
}

#[should_panic="Input wires not set"]
#[test]
fn panics_if_input_wires_not_set() {
    let mut circuit_builder = CircuitBuilder::new();
    circuit_builder.get_circuit_build();
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

#[test]
fn branches_assigned_correctly_for_two_ifs_with_adders() {
    let cb = get_nested_if_build_with_adder(); 
    let gates = cb.get_gates();
    
    // // Input gates has a single branch
    // for gate in input_gates {
    //     assert_eq!(gate.branches().len(), 1)
    // }

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
    builder.set_input_wires(1); // Need to set to avoid failing
    
    let inputs = builder.build_input_wires(4); 
    let cond = &inputs[0];
    let wi = &inputs[1];
    let wj = &inputs[2];
    let wz = &inputs[3];

    // First if: 
    let and_0 = builder.build_and(wi, wi);
    let and_1 = builder.build_and(wj, wj);
    let if_out = builder.build_if(cond, &and_0, &and_1);

    // Second if, nested
    let and_2 = builder.build_and(&wz, &wz);
    builder.build_if(cond, &if_out, &and_2);

    let cb = builder.get_circuit_build();
    let gates = cb.get_gates();

    let id_to_gate_index: HashMap<BigUint, usize> = gates.iter().enumerate().map(|(idx, gate)| (gate.wo().wire_id().clone(), idx)).collect();
    let and_0_build = &gates[id_to_gate_index.get(and_0[0].wire_id()).unwrap().clone()];
    let and_1_build = &gates[id_to_gate_index.get(and_1[0].wire_id()).unwrap().clone()];
    let and_2_build = &gates[id_to_gate_index.get(and_2[0].wire_id()).unwrap().clone()];

    (cb.clone(), vec![and_0_build.clone(), and_1_build.clone(), and_2_build.clone()])
}

fn get_nested_if_build_with_adder() -> CircuitBuild {
    let mut builder = CircuitBuilder::new();
    builder.set_input_wires(1);

    let garbler_input = 7.to_biguint().unwrap();
    let evaluator_input = 23.to_biguint().unwrap();
    
    let inputs = builder.build_input_wires(1); 
    let cond = &inputs[0];
    let required_bits = max(garbler_input.bits(), evaluator_input.bits());

    let mut circuit_builder = CircuitBuilder::new();
    let input_wires_garbler = circuit_builder.build_input_wires(required_bits as u32);
    let input_wires_evaluator = circuit_builder.build_input_wires(required_bits as u32);

    // First if: 
    let adder_0 = builder.build_adder(&input_wires_garbler, &input_wires_garbler); // garbler_number + garbler_number
    let adder_1 = builder.build_adder(&input_wires_evaluator, &input_wires_evaluator); // evaluator_number + evaluator_number
    let if_out = builder.build_if(cond, &adder_0, &adder_1);

    // Second if, nested
    let adder_2 = builder.build_adder(&if_out, &input_wires_evaluator); // 2 * garbler_number + evaluator_number
    builder.build_if(cond, &adder_2, &input_wires_evaluator); 

    let cb = builder.get_circuit_build();
    let gates = cb.get_gates();

    let id_to_gate_index: HashMap<BigUint, usize> = gates.iter().enumerate().map(|(idx, gate)| (gate.wo().wire_id().clone(), idx)).collect();
    for input_wire in input_wires_evaluator {

    }
    // let and_0_build = &gates[id_to_gate_index.get(adder_0[].wire_id()).unwrap().clone()];
    // let and_1_build = &gates[id_to_gate_index.get(adder_1[].wire_id()).unwrap().clone()];
    // let and_2_build = &gates[id_to_gate_index.get(adder_2[].wire_id()).unwrap().clone()  
    cb.clone()
}