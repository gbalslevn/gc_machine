
use crate::{evaluator::{original_evaluator::OriginalEvaluator}, garbler::Garbler, gates::{gate_gen::GateGen, original_gate_gen::OriginalGateGen}, peer::Peer, websocket, wires::{original_wire_gen::OriginalWireGen, wire_gen::WireGen}};
use std::time::Duration;
use libp2p::{PeerId};

#[tokio::test]
// When it says hello, the other party replies with hello
async fn can_send_and_receive_hello_query() {
    let wire_gen = OriginalWireGen::new();
    let gate_gen = OriginalGateGen::new(wire_gen.clone());
    let garbler = Garbler::new(gate_gen, wire_gen);
    let evaluator = OriginalEvaluator::new();
    let client_a = Peer::new(garbler, evaluator).await;

    let client_b = websocket::run().await.expect("Could not start client_b");
    client_b.dial(client_a.get_address()).await.expect("Dialing failed");
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    let response = client_b.send_query(client_a.get_peer_id(), websocket::Query::Hello).await.expect("send_message failed");
    assert_eq!(response, websocket::Response::Greeting("Hello".to_string()))
}

#[tokio::test]
async fn cannot_send_msg_to_unconnected_peer_id() {
    
    let client_a = websocket::run().await.expect("Could not start client_a");
    
    let unconnected_peer_id = PeerId::random();
    
    let response = client_a.send_query(unconnected_peer_id, websocket::Query::Hello).await;
    
    if let Err(e) = response {
        let err_string = format!("{:?}", e);
        assert!(err_string.contains("NotConnected"))
    }
}