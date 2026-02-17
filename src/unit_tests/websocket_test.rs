
use crate::websocket::{self};
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn socket_can_return_the_last_sent_msg() {
    let socket_address = "127.0.0.1:12346".to_string();
    let receiver_socket_client = websocket::run(socket_address.clone()).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await; 
    let sender_socket_client = websocket::run(socket_address.clone()).await;  
    
    let msg_1 = Message::text(format!("msg1"));
    sender_socket_client.send_message(msg_1).await;
    let msg_2 = Message::text(format!("msg2"));
    sender_socket_client.send_message(msg_2).await;
    
    tokio::time::sleep(std::time::Duration::from_millis(50)).await; 
    let received_msg = receiver_socket_client.get_last_msg().await;
    
    assert!(received_msg.to_text().unwrap() == "msg2")
}

#[tokio::test]
async fn socket_can_listen_for_a_new_message() {
    let receiver_socket_client = websocket::run("localhost:12345".to_string()).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await; 
    let sender_socket_client = websocket::run("localhost:12345".to_string()).await;
    
    let msg = Message::text(format!("msg"));
    sender_socket_client.send_message(msg).await;

    let received_msg = receiver_socket_client.listen_for_next_msg().await;

    assert!(received_msg.to_text().unwrap() == "msg");
}

// should test more