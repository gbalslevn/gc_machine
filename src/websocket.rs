use std::{error::Error, io, time::Duration};
use futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
use libp2p::{Multiaddr, PeerId, Stream, StreamProtocol, Swarm, tcp, tls, yamux};
use libp2p_stream::{self as stream};
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

const MSG_PROTOCOL: StreamProtocol = StreamProtocol::new("/msg");

pub enum SwarmCmd {
    Dial(Multiaddr),
}

pub enum Query {
    Hello,
}

#[derive(Clone)]
pub struct PeerClient {
    control: stream::Control,
    peer_id: PeerId, 
    address : Multiaddr,
    swarm_control: tokio::sync::mpsc::Sender<SwarmCmd>,
}

impl PeerClient {
    pub fn new(control: stream::Control, peer_id : PeerId, address : Multiaddr, swarm_control : tokio::sync::mpsc::Sender<SwarmCmd>) -> Self {
        Self { control, peer_id, address, swarm_control }
    }
    pub fn get_peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn get_address(&self) -> Multiaddr {
        self.address.clone()
    }

    pub async fn dial(&self, addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm_control.send(SwarmCmd::Dial(addr)).await?;
        Ok(())
    }

    /// sends a message and gets a response
    pub async fn send_message(&mut self, peer: PeerId, data: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut stream = self.control.open_stream(peer, MSG_PROTOCOL).await?;

        stream.write_all(&data).await?; 
        stream.close().await?;
        let mut response = Vec::new();
        stream.read_to_end(&mut response).await?;

        Ok(response)
    }
}

// Creates a new peer
pub async fn new() -> Result<PeerClient, Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .try_init();

    let mut swarm = create_swarm()?;
    let peer_id = *swarm.local_peer_id();
    let mut control = swarm.behaviour().new_control();
    
    let mut incoming_streams = control.accept(MSG_PROTOCOL).unwrap();
    // Setup handle requests
    tokio::spawn(async move {
        while let Some((peer, stream)) = incoming_streams.next().await {
            tokio::spawn(async move {
                if let Err(e) = handle_request(stream).await {
                    tracing::warn!(%peer, "Handler error: {e}");
                }
            });
        }
    });
    
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
    let client = PeerClient::new(control.clone(), peer_id, listen_addr, cmd_sender);

    Ok(client)
}

// Req and reply
async fn handle_request(mut stream: Stream) -> io::Result<()> {
    let mut request_data = Vec::new();
    stream.read_to_end(&mut request_data).await?;
    let msg = String::from_utf8_lossy(&request_data);
    
    tracing::info!("Received Request: {}", msg);
    let response = format!("Hello");
    stream.write_all(response.as_bytes()).await?;
    stream.close().await?;

    Ok(())
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