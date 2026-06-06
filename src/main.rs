use circuit_macro::{circuit, circuit_fn};
use gc_machine::{circuit_builder::{CircuitBuilder}, evaluator::{half_gates_evaluator::HalfGatesEvaluator}, garbler::Garbler, gates::{gate_gen::GateGen, half_gates_gate_gen::HalfGatesGateGen}, peer::Peer};
use num_bigint::ToBigUint;

#[tokio::main]
async fn main() {
    let garbler = Garbler::new(HalfGatesGateGen::new());
    let evaluator = HalfGatesEvaluator::new();
    let peer = Peer::new(garbler, evaluator).await;
    println!("Started peer {}", peer.get_peer_id());
    println!("Listening on: {}", peer.get_address());
    let cb = circuit!(produce_build);
    let input = 7.to_biguint().unwrap();
    peer.setup_circuit_context(&input, &cb, &cb.required_input_bits).await;
    println!("Ready. Waiting for connection...");

    // Keep alive 
    tokio::signal::ctrl_c().await.unwrap();
}

#[circuit_fn]
fn produce_build(garbler_input: usize, evaluator_input: usize) -> usize {
    garbler_input + evaluator_input
}
