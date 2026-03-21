use std::{collections::HashMap, error::Error, time::Duration};
use futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
use k256::PublicKey;
use libp2p::{Multiaddr, PeerId, StreamProtocol, Swarm, tcp, tls, yamux};
use libp2p_stream::{self as stream};
use num_bigint::BigUint;
use tracing_subscriber::{EnvFilter, filter::LevelFilter};
use serde::{Serialize, Deserialize};

use crate::ot::eg_elliptic::CipherText;

pub enum SwarmCmd {
    Dial(Multiaddr),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Query {
    Hello,
    ExecuteProtocol,
    EvalGC(Vec<Vec<BigUint>>, Vec<BigUint>, HashMap<BigUint, BigUint>, HashMap<BigUint, (CipherText, CipherText)>, [(BigUint, u8); 2]), 
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Response {
    Greeting(String),
    ProvideEvalInput(Vec<[PublicKey; 2]>), 
    ProvideGCResult(u8)
}

#[derive(Clone)]
pub struct SocketClient {
    pub control: stream::Control,
    peer_id: PeerId, 
    pub address : Multiaddr,
    pub swarm_control: tokio::sync::mpsc::Sender<SwarmCmd>,
    pub protocol : StreamProtocol
}

impl SocketClient {
    pub fn new(control: stream::Control, peer_id : PeerId, address : Multiaddr, swarm_control : tokio::sync::mpsc::Sender<SwarmCmd>) -> Self {
        let protocol: StreamProtocol = StreamProtocol::new("/msg");

        Self {control, peer_id, address, swarm_control, protocol}
    }

    pub fn get_control(&self) -> stream::Control {
        self.control.clone()
    }

    pub fn get_protocol(&self) -> StreamProtocol {
        self.protocol.clone()
    }

    pub fn get_address(&self) -> Multiaddr {
        self.address.clone()
    }

    pub fn get_peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub async fn dial(&self, addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm_control.send(SwarmCmd::Dial(addr)).await?;
        Ok(())
    }

    /// sends a message and gets a response
    pub async fn send_query(&self, peer: PeerId, query: Query) -> Result<Response, Box<dyn Error>> {
        let mut stream = self.control.clone().open_stream(peer, self.get_protocol()).await?;

        let request_bytes = postcard::to_allocvec(&query)?;
        stream.write_all(&request_bytes).await?; 
        stream.close().await?;

        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).await?;
        let response: Response = postcard::from_bytes(&response_bytes)?;

        Ok(response)
    }
}

pub async fn run() -> Result<SocketClient, Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .try_init();

    let mut swarm = create_swarm()?;
    let peer_id = *swarm.local_peer_id();
    let control = swarm.behaviour().new_control();
    
    let (addr_sender, addr_receiver) = tokio::sync::oneshot::channel(); // A way to save our address
    let mut addr_sender = Some(addr_sender);
    let (cmd_sender, mut cmd_receiver) = tokio::sync::mpsc::channel::<SwarmCmd>(8); // To send commands to the swarm like dial
    
    // Listen for connections, move into background
    let address = "/ip4/0.0.0.0/tcp/0";
    swarm.listen_on(address.parse()?)?;
    tokio::spawn(async move {
        loop {
            tokio::select! {
                // Listen for commands from the PeerClient
                Some(cmd) = cmd_receiver.recv() => {
                    match cmd {
                        SwarmCmd::Dial(addr) => {
                            if let Err(e) = swarm.dial(addr) {
                                tracing::error!("Dial failed: {:?}", e);
                            }
                        }
                    }
                }
                event = swarm.select_next_some() => {
                    if let libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } = event {
                        if let Some(s) = addr_sender.take() {
                            let _ = s.send(address);
                        }
                    }
                }
            }
        }
    });

    let listen_addr = tokio::time::timeout(Duration::from_secs(2), addr_receiver).await??;
    let client = SocketClient::new(control.clone(), peer_id, listen_addr, cmd_sender);

    Ok(client)
}

// Creates a node
fn create_swarm() -> Result<Swarm<stream::Behaviour>, Box<dyn Error>> {
    let swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            tls::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| stream::Behaviour::new())?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(10)))
        .build();
    Ok(swarm)
}