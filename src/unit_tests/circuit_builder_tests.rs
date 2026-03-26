use std::{cmp::max, collections::{HashMap, HashSet}};
use num_bigint::{BigUint, ToBigUint};
use crate::circuit_builder::{CircuitBuild, CircuitBuilder, GateBuild};

#[test]
fn gates_are_sorted_by_increasing_output_layer() {
    let mut circuit_builder = CircuitBuilder::new();
    let (input_a, input_b) = circuit_builder.set_input_wires(10);
    circuit_builder.build_is_equal(&input_a, &input_b);
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
fn one_if_creates_2_branches() {
    let (cb, _input_gates) = get_if_build(); 
    let gates = cb.get_gates();

    // We expect all branches to be present for the last gate as the output could have come from all branches
    assert_eq!(gates[gates.len() - 1].branches().len(), 2);
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
// Tests a specific situiation assigns the branches correctly
fn branches_assigned_correctly_for_two_ifs_with_adders() {
    let (_cb, adder_0_gates, adder_1_gates, adder_2_gates, cond_gate) = get_nested_if_build_with_adder(); 
    
    // cond gate should have all branches as its used for all branches
    assert_eq!(cond_gate.branches().len(), 3);

    // Adder 0 should have only branch 2
    for gate in adder_0_gates {
        let branches : HashSet<usize> = vec![0].into_iter().collect();
        assert_eq!(gate.branches(), &branches);
    }
    
    // Adder 1 should have branch 1
    for gate in adder_1_gates {
        let branches : HashSet<usize> = vec![1].into_iter().collect();
        assert_eq!(gate.branches(), &branches);
    }
    
    // Adder 2 should have branch 1, 2
    for gate in adder_2_gates {
        let branches : HashSet<usize> = vec![0, 1].into_iter().collect();
        assert_eq!(gate.branches(), &branches);
    }
}

fn get_if_build() -> (CircuitBuild, Vec<GateBuild>) {
    let mut builder = CircuitBuilder::new();
    builder.set_input_wires(1); // Need to set to avoid failing
    
    let inputs = builder.build_input_wires(3); 
    let cond = &inputs[0];
    let wi = &inputs[1];
    let wj = &inputs[2];

    let and_0 = builder.build_and(wi, wi);
    let and_1 = builder.build_and(wj, wj);
    let _if_out = builder.build_if(cond, &and_0, &and_1);

    let cb = builder.get_circuit_build();
    let gates = cb.get_gates();

    let id_to_gate_index: HashMap<BigUint, usize> = gates.iter().enumerate().map(|(idx, gate)| (gate.wo().wire_id().clone(), idx)).collect();
    let and_0_build = &gates[id_to_gate_index.get(and_0[0].wire_id()).unwrap().clone()];
    let and_1_build = &gates[id_to_gate_index.get(and_1[0].wire_id()).unwrap().clone()];

    (cb.clone(), vec![and_0_build.clone(), and_1_build.clone()])
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

fn get_nested_if_build_with_adder() -> (CircuitBuild, Vec<GateBuild>, Vec<GateBuild>, Vec<GateBuild>, GateBuild) {
    let mut builder = CircuitBuilder::new();

    let garbler_input = 7.to_biguint().unwrap();
    let evaluator_input = 23.to_biguint().unwrap();
    
    let inputs = builder.build_input_wires(1); 
    let cond_wire = inputs[0].clone();
    let cond = builder.build_is_equal(&vec![cond_wire.clone()], &vec![cond_wire]);
    let required_bits = max(garbler_input.bits(), evaluator_input.bits());

    let (input_wires_garbler, input_wires_evaluator) = builder.set_input_wires(required_bits);

    // First if: 
    let adder_0 = builder.build_adder(&input_wires_garbler, &input_wires_garbler); // garbler_number + garbler_number
    let adder_1 = builder.build_adder(&input_wires_evaluator, &input_wires_evaluator); // evaluator_number + evaluator_number
    let if_out = builder.build_if(&cond, &adder_0, &adder_1);

    // Second if, nested
    let adder_2 = builder.build_adder(&if_out, &input_wires_evaluator); // 2 * garbler_number + evaluator_number
    builder.build_if(&cond, &adder_2, &input_wires_evaluator); 

    
    let cb = builder.get_circuit_build();
    let gates = cb.get_gates();

    let mut adder_0_gates = vec![];
    let mut adder_1_gates = vec![];
    let mut adder_2_gates = vec![];
    let id_to_gate_index: HashMap<BigUint, usize> = gates.iter().enumerate().map(|(idx, gate)| (gate.wo().wire_id().clone(), idx)).collect();
    for wire in adder_0 {
        adder_0_gates.push(gates[id_to_gate_index.get(wire.wire_id()).unwrap().clone()].clone())
    }
    for wire in adder_1 {
        adder_1_gates.push(gates[id_to_gate_index.get(wire.wire_id()).unwrap().clone()].clone())
    }
    for wire in adder_2 {
        adder_2_gates.push(gates[id_to_gate_index.get(wire.wire_id()).unwrap().clone()].clone())
    }
    let cond_gate = &gates[id_to_gate_index.get(cond.wire_id()).unwrap().clone()];
    (cb.clone(), adder_0_gates, adder_1_gates, adder_2_gates, cond_gate.clone())
}