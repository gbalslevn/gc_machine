use futures_util::SinkExt;
use futures_util::{StreamExt, stream::SplitSink, stream::SplitStream};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, accept_async, client_async_tls,
};

// Websocket using tokio-tungstenite
// https://crates.io/crates/tokio-tungstenite

#[derive(Debug)]
pub enum SocketCommand {
    GetRxMsgCount(futures_channel::oneshot::Sender<usize>),
    SendMessage(Message),
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
}

// Starts websocket on provided address and returns a SocketClient which can be used to communicate with the socket. 
pub async fn run(address: String) -> SocketClient {
    let (internal_tx, mut internal_rx) = mpsc::channel::<SocketCommand>(32);
    let async_socket_logic = async move {
        let (mut socket_tx, mut socket_rx) = connect(address).await; // Tries to connect to the socket address. If no channel exists, it starts its own socket and listens for connections. 
        let mut msg_counter = 0;
        loop {
            tokio::select! { // Waits on multiple concurrent branches
                // Listens for messages from the socket
                Some(Ok(msg)) = socket_rx.next() => {
                    msg_counter += 1;
                    // internal_tx.send(msg).await.unwrap(); // Forward message to handle it
                    println!("Received network message: {:?}", msg);
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
                    }
                }
            }
        }
    };
    tokio::spawn(async_socket_logic); // Spawns the async task as a thread 
    SocketClient::new(internal_tx)
}

// Tries to connect to a Socket on the given address. If unsuccesfull it starts its own Socket on the address and listens for connections
async fn connect(address: String) -> (SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>) {
    let (socket_tx, socket_rx);
    // Try to connect to the address, if address does not exist, create own Socket. P2P paradigm
    match TcpStream::connect(&address).await {
        Ok(tcp) => {
            // A Socket existed and we connect to it 
            let url = format!("ws://{}", address);
            let (stream, _) = client_async_tls(&url, tcp)
                .await
                .expect(&format!("Client failed to connect on {}", &url));
            // Tcp is normally full-duplex, meaning data can flow in both directions, but in rust a peer owns both the read half and write half of the stream. There is a ownership problem. The borrow checker problem. You would therefore not be able to listen for messages while you send messages. To fix this we need to split the stream such that a peer can both read and write at the same time.
            (socket_tx, socket_rx) = stream.split();
        }
        Err(_) => {
            // We need to create our own Socket which listens for a connection
            let listener = TcpListener::bind(&address).await.unwrap();
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
