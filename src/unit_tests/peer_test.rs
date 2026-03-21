use std::{cmp::max, time::Duration};

use num_bigint::ToBigUint;

use crate::{circuit_builder::CircuitBuilder, evaluator::original_evaluator::OriginalEvaluator, garbler::Garbler, gates::{gate_gen::GateGen, original_gate_gen::OriginalGateGen}, peer::Peer, wires::{original_wire_gen::OriginalWireGen, wire_gen::WireGen}};

#[tokio::test]
#[should_panic]
// After a protocol has been executed the circuit context should be empty
async fn error_if_context_not_setup() {
    // Create two peers which connects to each other
    let wire_gen = OriginalWireGen::new();
    let gate_gen = OriginalGateGen::new(wire_gen.clone());
    let evaluator = OriginalEvaluator::new();
    let garbler = Garbler::new(gate_gen, wire_gen);
    let peer_a = Peer::new(garbler, evaluator).await;

    let wire_gen = OriginalWireGen::new();
    let gate_gen = OriginalGateGen::new(wire_gen.clone());
    let evaluator = OriginalEvaluator::new();
    let garbler = Garbler::new(gate_gen, wire_gen);
    let peer_b = Peer::new(garbler, evaluator).await;
    
    peer_a.connect(peer_b.get_address()).await.expect("Could not connect");
    tokio::time::sleep(Duration::from_millis(200)).await; // Wait for it to connect

    // Create a circuit build which both peers in some way agree on
    let garbler_input = 12.to_biguint().unwrap();
    let evaluator_input = 12.to_biguint().unwrap();
    let required_bits = max(&garbler_input, &evaluator_input).bits(); // They somehow know the max amount of bits needed 
    let mut builder = CircuitBuilder::new();
    let input_wires = builder.build_input_wires((required_bits * 2) as u32);
    builder.build_is_equal(input_wires);
    let cb = builder.get_circuit_build();

    // Before execution, circuit context should be empty
    let response = peer_a.execute_protocol(peer_b.get_peer_id()).await;
    let error = response.expect_err("Expected protocol to fail, but it succeded");
    assert!(error.to_string().contains("Circuit context not set"));
    
    peer_a.setup_circuit_context(garbler_input, cb.clone(), required_bits).await;
    peer_b.setup_circuit_context(evaluator_input, cb, required_bits).await;
    
    let _result = peer_a.execute_protocol(peer_b.get_peer_id()).await;
    
    // After execution, circuit context should be empty
    let response = peer_a.execute_protocol(peer_b.get_peer_id()).await;
    let error = response.expect_err("Expected protocol to fail, but it succeded");
    assert!(error.to_string().contains("Circuit context not set"));
}