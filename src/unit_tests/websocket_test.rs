
use crate::websocket::{self, SocketClient};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

#[tokio::test]
async fn socket_can_return_the_last_sent_msg() {
    let config = websocket::SocketConfig::new("localhost:12345".to_string());

    let receiver_socket_client = websocket::run(&config).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await; 
    let sender_socket_client = websocket::run(&config.as_client()).await; 
    let is_connected = is_connected(&receiver_socket_client, &sender_socket_client).await;
    assert!(is_connected);
    
    let msg_1 = Message::text(format!("msg1"));
    sender_socket_client.send_message(msg_1).await;
    let msg_2 = Message::text(format!("msg2"));
    sender_socket_client.send_message(msg_2).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await; 
    
    let received_msg = receiver_socket_client.get_last_msg().await;
    
    assert!(received_msg.to_text().unwrap() == "msg2")
}

#[tokio::test]
async fn socket_can_listen_for_a_new_message() {
    let config = websocket::SocketConfig::new("localhost:12346".to_string());
    let receiver_socket_client = websocket::run(&config).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await; 
    let sender_socket_client = websocket::run(&config.as_client()).await;
    
    let msg = Message::text(format!("msg"));
    sender_socket_client.send_message(msg).await;

    let received_msg = receiver_socket_client.listen_for_next_msg().await;

    assert!(received_msg.to_text().unwrap() == "msg");
}

// #[tokio::test]
// #[should_panic(expected = "Invalid socket address")]
// async fn invalid_address_to_socket_throws_error() {
//     let invalid_address = "gustav".to_string();
//     let config = SocketConfig::new(invalid_address);
//     let _socket_client = websocket::run(config).await;
// }

// #[tokio::test]
// #[should_panic(expected = "Socket has no connections")]
// async fn sending_a_message_without_being_connected_throws_error() {
//     let config = SocketConfig::new(SOCKET_ADDRESS.to_string());
//     tokio::time::sleep(std::time::Duration::from_millis(50)).await; 
//     let sender_socket_client = websocket::run(config).await;
    
//     let msg = Message::text(format!("msg"));
//     sender_socket_client.send_message(msg).await;
// }

// Sends a message back and forth between the two to ensure they are connected
async fn is_connected(client1 : &SocketClient, client2 : &SocketClient) -> bool {
    let token = Uuid::new_v4().to_string();
    client1.send_message(Message::text(&token)).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await; 
    let msg = client2.get_last_msg().await;
    msg.to_text().unwrap() == token
}