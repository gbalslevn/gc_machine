use futures_util::SinkExt;
use futures_util::{StreamExt, stream::SplitSink, stream::SplitStream};
use std::collections::VecDeque;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, accept_async, client_async_tls};

// Websocket using tokio-tungstenite
// https://crates.io/crates/tokio-tungstenite

#[derive(Debug)]
pub enum SocketCommand {
    GetRxMsgCount(futures_channel::oneshot::Sender<usize>),
    SendMessage(Message),
    HandleMessage(Message),
    Listen(futures_channel::oneshot::Sender<Message>),
    GetLastMsg(futures_channel::oneshot::Sender<Message>),
}
#[derive(Clone)]
pub struct SocketConfig {
    address: String,
    force_client_connection: bool,
}

impl SocketConfig {
    pub fn new(address: String) -> Self {
        // if address.parse::<SocketAddr>().is_err() {
        //     panic!("Invalid socket address");
        // } 
        Self {
            address,
            force_client_connection: false,
        }
    }
    // Ensures socket connects to a listening socket and does not create its own.
    pub fn as_client(mut self) -> Self {
        self.force_client_connection = true;
        self
    }
    // webpki uses certs from web. Can also use native certs, which is certs on the users machine. Could run into problems if no cert is on the machine. 
}

pub struct SocketClient {
    tx: mpsc::Sender<SocketCommand>,
}

// Contains method for interacting with the socket. It recognizes commands as enum of SocketCommand
impl SocketClient {
    pub fn new(tx: mpsc::Sender<SocketCommand>) -> Self {
        Self { tx }
    }
    pub async fn get_rx_msg_count(&self) -> usize {
        let (reply_tx, reply_rx) = futures_channel::oneshot::channel(); // A channel to send a single value async from one thread to another

        self.tx
            .send(SocketCommand::GetRxMsgCount(reply_tx))
            .await
            .expect("Socket task died");

        let result = reply_rx.await.expect("Socket dropped the reply channel");
        result
    }
    pub async fn send_message(&self, msg: Message) {
        self.tx
            .send(SocketCommand::SendMessage(msg))
            .await
            .expect("Socket task died");
    }
    // Listens for the next message and returns it
    pub async fn listen_for_next_msg(&self) -> Message {
        let (reply_tx, reply_rx) = futures_channel::oneshot::channel();
        self.tx
            .send(SocketCommand::Listen(reply_tx))
            .await
            .expect("Socket task died");
        let result = reply_rx.await.expect("Socket dropped the reply channel");
        result
    }
    pub async fn get_last_msg(&self) -> Message {
        let (reply_tx, reply_rx) = futures_channel::oneshot::channel(); // A channel to send a single value async from one thread to another

        self.tx
            .send(SocketCommand::GetLastMsg(reply_tx))
            .await
            .expect("Socket task died");

        let result = reply_rx.await.expect("Socket dropped the reply channel");
        result
    }
}

// Starts websocket on provided address and returns a SocketClient which can be used to communicate with the socket.
pub async fn run(config: &SocketConfig) -> SocketClient {
    let (internal_tx, mut internal_rx) = mpsc::channel::<SocketCommand>(32);
    let config = config.clone();
    let async_socket_logic = async move {
        let (mut socket_tx, mut socket_rx) = connect(config.address, config.force_client_connection).await; // Tries to connect to the socket address. If no channel exists, it starts its own socket and listens for connections. 
        let mut msg_counter = 0;
        let mut message_queue = VecDeque::<Message>::new();
        let mut listening_channel: Option<futures_channel::oneshot::Sender<Message>> = None;
        loop {
            tokio::select! { // Waits on multiple concurrent branches
                // Listens for messages from the socket
                Some(Ok(msg)) = socket_rx.next() => {
                    msg_counter += 1;
                    if let Some(channel) = listening_channel.take() { // For now we only have oneshot listening. Maybe we should also have listening where you can get multiple messages
                        let _ = channel.send(msg);
                    } else {
                        // internal_tx.send(SocketCommand::HandleMessage(msg)).await.unwrap();
                        message_queue.push_back(msg.clone());
                    }
                }
                // Forwards messages from internal_rx. The internal channel for the Socket itself.
                Some(cmd) = internal_rx.recv() => {
                    // Internal methods to control the Socket
                    match cmd {
                        SocketCommand::GetRxMsgCount(reply_channel) => {
                        let _ = reply_channel.send(msg_counter);
                        }
                        SocketCommand::SendMessage(msg) => { // sends message to other party
                            socket_tx.send(msg).await.unwrap();
                        }
                        SocketCommand::HandleMessage(msg) => {
                            message_queue.push_back(msg.clone());
                        }
                        SocketCommand::Listen(reply_channel) => {
                            println!("Started listening");
                            listening_channel = Some(reply_channel);
                        }
                        SocketCommand::GetLastMsg(reply_channel) => {
                            let _ = reply_channel.send(message_queue.pop_back().expect("Could not get the latest message from queue"));
                        }
                    }
                }
            }
        }
    };
    tokio::spawn(async_socket_logic); // Spawns the async task as a thread 
    SocketClient::new(internal_tx)
}

// Tries to connect to a Socket on the given address. If unsuccesfull it starts its own Socket on the address and listens for connections
async fn connect(
    address: String,
    connect_with_force: bool,
) -> (
    SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
) {
    let (socket_tx, socket_rx);
    // Try to connect to the address, if address does not exist, create own Socket. P2P paradigm
    match TcpStream::connect(&address).await {
        Ok(tcp) => {
            // A Socket existed and we connect to it
            let url = format!("ws://{}", address); // use wss:// if we want tls
            let (stream, _) = client_async_tls(&url, tcp)
                .await
                .expect(&format!("Client failed to connect on {}", &url));
            // Tcp is normally full-duplex, meaning data can flow in both directions, but in rust a peer owns both the read half and write half of the stream. There is a ownership problem. The borrow checker problem. You would therefore not be able to listen for messages while you send messages. To fix this we need to split the stream such that a peer can both read and write at the same time.
            (socket_tx, socket_rx) = stream.split();
        }
        Err(_) => {
            if connect_with_force {
                panic!("Could not connect to socket on {}", address);
            }
            // We need to create our own Socket which listens for a connection
            let listener = TcpListener::bind(&address).await.expect("Could not start new socket. Perhaps address is invalid or another peer is in process of starting on the same address.");
            let (connection, _) = listener.accept().await.expect("No connections to accept");
            let maybe_tls = MaybeTlsStream::Plain(connection);
            let stream = accept_async(maybe_tls).await;
            let stream = stream.expect("Failed to handshake with connection"); // Test the stream was established
            match stream.get_ref() {
                MaybeTlsStream::Plain(_) => println!("Warning: Unencrypted connection!"),
                MaybeTlsStream::Rustls(_) => println!("Success: Connection is encrypted."),
                _ => println!("Other TLS provider used."),
            }
            (socket_tx, socket_rx) = stream.split();
        }
    }
    (socket_tx, socket_rx)
}


// Great discusssion on tls 
// https://github.com/snapview/tungstenite-rs/issues/127
