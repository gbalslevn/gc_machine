
use crate::websocket::{self};
use futures_util::{StreamExt};
use tokio::{
    io::{AsyncRead, AsyncWrite},
};
use tokio_tungstenite::{WebSocketStream};

// async fn run_connection<S>(
//     connection: WebSocketStream<S>,
//     msg_tx: futures_channel::oneshot::Sender<Vec<Message>>,
// ) where
//     S: AsyncRead + AsyncWrite + Unpin,
// {
//     println!("Running connection");
//     let mut connection = connection;
//     let mut messages = vec![];
//     while let Some(message) = connection.next().await {
//         println!("Message received");
//         let message = message.expect("Failed to get message");
//         messages.push(message);
//     }
//     msg_tx.send(messages).expect("Failed to send results");
// }

// #[tokio::test]
// async fn server_can_communicate_without_a_websocket() {
//     // Channel for testing. The channel is shared memory therefore it only works within a single program.
//     let (con_tx, con_rx) = futures_channel::oneshot::channel(); 
//     let (msg_tx, msg_rx) = futures_channel::oneshot::channel::<Vec<Message>>(); 

//     let f = async move {
//         let listener = TcpListener::bind("127.0.0.1:12346").await.unwrap();
//         println!("Server ready");
//         con_tx.send(()).unwrap();
//         println!("Waiting on next connection");
//         let (connection, _) = listener.accept().await.expect("No connections to accept");
//         let stream = accept_async(connection).await;
//         let stream: WebSocketStream<TcpStream> = stream.expect("Failed to handshake with connection");
//         run_connection(stream, msg_tx).await;
//     };

//     tokio::spawn(f); // Spawns the asynchronus task of the server being ready and then we wait for server to be ready
//     println!("Waiting for server to be ready");

//     con_rx.await.expect("Server not ready"); // Client listens on the channel (the shared memory) to know when server has started. Then it can connect. In real life it would just connect and server should be up.
//     let tcp = TcpStream::connect("127.0.0.1:12346").await.expect("Failed to connect");
//     let url = "ws://localhost:12345/";
//     let (stream, _) = client_async_tls(url, tcp).await.expect("Client failed to connect");
//     // Tcp is normally full-duplex, meaning data can flow in both directions, but in rust a peer owns both the read half and write half of the stream. There is a ownership problem. The borrow checker problem. You would therefore not be able to listen for messages while you send messages. To fix this we need to split the stream such that a peer can both read and write at the same time. 
//     let (mut tx, _rx) = stream.split(); 
//     for i in 1..10 {
//         println!("Sending message");
//         tx.send(Message::text(format!("{}", i))).await.expect("Failed to send message");
//     }

//     tx.close().await.expect("Failed to close");

//     println!("Waiting for response messages");
//     let messages = msg_rx.await.expect("Failed to receive messages");
//     assert_eq!(messages.len(), 10);
// }



// I need to make a lot of tests. Maybe error messages

