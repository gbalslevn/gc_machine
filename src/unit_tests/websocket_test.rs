
use crate::websocket;
use std::time::Duration;
use libp2p::{PeerId};

#[tokio::test]
// When it says hello, the other party replies with hello
async fn can_send_and_receive_reply() {
    let SAY_HELLO_ENUM = b"SAY_HELLO".to_vec();
    let client_a = websocket::new().await.expect("Could not start client_a");

    let mut client_b = websocket::new().await.expect("Could not start client_b");
    client_b.dial(client_a.get_address()).await.expect("Dialing failed");
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    let response = client_b.send_message(client_a.get_peer_id(), SAY_HELLO_ENUM).await.expect("send_message failed");
    assert_eq!(String::from_utf8_lossy(&response), "Hello")
}

#[tokio::test]
async fn cannot_send_msg_to_unconnected_peer_id() {
    let SAY_HELLO_ENUM = b"SAY_HELLO".to_vec();
    let mut client_a = websocket::new().await.expect("Could not start client_a");
    
    let unconnected_peer_id = PeerId::random();
    
    let response = client_a.send_message(unconnected_peer_id, SAY_HELLO_ENUM).await;
    
    if let Err(e) = response {
        let err_string = format!("{:?}", e);
        assert!(err_string.contains("NotConnected"))
    }
}