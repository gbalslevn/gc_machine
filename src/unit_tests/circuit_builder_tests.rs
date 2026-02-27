use std::ops::Add;

use num_bigint::ToBigUint;
use uuid::Uuid;

use crate::{circuit_builder::{self, WireBuild}, gates::gates::GateType};

#[test] 
fn create_xor_increment_layer_twice() {
    let wi_input_layer = 23.to_biguint().unwrap();
    let wi = WireBuild::new(wi_input_layer.clone());

    let wj_input_layer = 41.to_biguint().unwrap();
    let wj = WireBuild::new( wj_input_layer.clone());

    let gate_calculated_layer = wi_input_layer.max(wj_input_layer);
    let or_circuit = circuit_builder::create_or(&wi, &wj);
    
    assert!(&gate_calculated_layer.add(2.to_biguint().unwrap()) == or_circuit[2].wo().output_layer());
}

#[test] 
fn create_gate_increments() {
    let wi_input_layer = 23.to_biguint().unwrap();
    let wi = WireBuild::new( wi_input_layer.clone());

    let wj_input_layer = 41.to_biguint().unwrap();
    let wj = WireBuild::new(wj_input_layer.clone());

    let layer_to_compute_gate = wi_input_layer.max(wj_input_layer);
    let gate = circuit_builder::create_gate(&wi, &wj, GateType::AND);

    assert!(&layer_to_compute_gate.add(1.to_biguint().unwrap()) == gate.wo().output_layer());
}

#[test]
fn create_gate_uses_correct_input() {
    let wi_input_layer = 23.to_biguint().unwrap();
    let wi = WireBuild::new( wi_input_layer.clone());

    let wj_input_layer = 41.to_biguint().unwrap();
    let wj = WireBuild::new( wj_input_layer.clone());

    let gate = circuit_builder::create_gate(&wi, &wj, GateType::AND);

    // assert!(gate.wi() == wi);
}