use std::error::Error;

use gc_machine::websocket_2;
use num_bigint::ToBigUint;

use futures::prelude::*;
use libp2p::{noise, ping, swarm::SwarmEvent, tcp, yamux, Multiaddr};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world :)");
    rustls::crypto::aws_lc_rs::default_provider().install_default().expect("Failed to install rustls crypto provider"); // Init for tls
    let yeast = 1.to_biguint().unwrap();
    let _eggs = yeast.clone();

    websocket_2::run().await
}
