
use crate::{evaluator::{original_evaluator::OriginalEvaluator}, garbler::Garbler, gates::{gate_gen::GateGen, original_gate_gen::OriginalGateGen}, peer::Peer, websocket, wires::{original_wire_gen::OriginalWireGen, wire_gen::WireGen}};
use std::time::Duration;
use libp2p::{Multiaddr, PeerId};

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

    let client = websocket::run().await.expect("Could not start client_a");
    
    let unconnected_peer_id = PeerId::random();
    
    let response = client.send_query(unconnected_peer_id, websocket::Query::Hello).await;
    
    if let Err(e) = response {
        let err_string = format!("{:?}", e);
        assert!(err_string.contains("NotConnected"))
    }
}

#[tokio::test]
async fn cannot_dial_to_unconnected_peer() {
    
    let client = websocket::run().await.expect("Could not start client_a");
    
    let unconnected_peer_id = PeerId::random();
    let fake_addr: Multiaddr = format!("/ip4/127.0.0.1/tcp/1234/p2p/{}", unconnected_peer_id).parse().unwrap();
    
    let response = client.dial(fake_addr).await;
    
    if let Err(e) = response {
        let err_string = format!("{:?}", e);
        assert!(err_string.contains("NotConnected"))
    }
}