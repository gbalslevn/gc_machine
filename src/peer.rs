use std::{cmp::max, error::Error, sync::Arc};

use futures::{AsyncReadExt, AsyncWriteExt, StreamExt, lock::Mutex};
use k256::{PublicKey, SecretKey};
use libp2p::{Multiaddr, PeerId, Stream};
use num_bigint::BigUint;

use crate::{circuit_builder::CircuitBuild, evaluator::{self, evaluator::Evaluator}, garbler::Garbler, gates::gate_gen::GateGen, websocket::{self, Query, Response, SocketClient}, wires::wire_gen::WireGen};

// Peer can both be a garbler and a evaluator
pub struct Peer<G : GateGen<W>, W : WireGen, E : Evaluator> {
    garbler : Mutex<Garbler<G, W>>, // we use a mutex to enable mutability without having a mutable self
    evaluator : Mutex<E>,
    socket : SocketClient,
    preparation : Mutex<Option<CircuitPrepeation>>  
}

#[derive(Clone)]
pub struct CircuitPrepeation {
    input : BigUint,
    build : CircuitBuild,
    required_bits : u64,
}

impl <G : GateGen<W>, W : WireGen, E : Evaluator> Peer<G, W, E> where 
    G: GateGen<W> + Send + Sync + 'static,
    W: WireGen + Send + Sync + 'static,
    E: Evaluator + Send + Sync + 'static, {

    pub async fn new(garbler : Garbler<G, W>, evaluator : E) -> Arc<Self> {
        let socket = websocket::run().await.expect("Failed to start socket");
        
        let peer = Arc::new(Peer { garbler : garbler.into(), evaluator : evaluator.into(), socket, preparation : None.into() });
        // need to spawn a copy of the peer with Arc which handles ownership and enables to call self inside a new thread
        peer.clone().spawn_query_handler().await;
        
        peer
    }

    pub async fn prepare_protocol(&self, input : BigUint, build : CircuitBuild, required_bits : u64) {
        {
            let preparation = CircuitPrepeation { input, build, required_bits };
            let mut current_preparation = self.preparation.lock().await;
            *current_preparation = Some(preparation);
     } 
    } 

    pub async fn connect(&self, address : Multiaddr) -> Result<(), Box<dyn Error>> {
        self.socket.dial(address).await
    }

    // Garbler executes protocol
    pub async fn execute_protocol(&self, peer: PeerId) -> Result<Response, Box<dyn Error>> {
        // let response = garbler_peer.execute_protocol(evaluator_peer.get_peer_id(), websocket::Query::GetGC((public_keys))).await.expect("send_message failed");
        let response = self.socket.send_query(peer, websocket::Query::ExecuteProtocol).await;
        if let Response::ProvideEvalInput(eval_input) = response {
            let circuit_preperation = self.get_preperation().await;
            let mut garbler = self.garbler.lock().await;
            let garbler_input =  garbler.create_circuit_input(&circuit_preperation.input, circuit_preperation.required_bits);
            let (garbled_gates, constant_wires, garbler_input, evaluator_input, conversion_table) = garbler.create_circuit(&circuit_preperation.build, &garbler_input, eval_input);

            self.socket.send_query(peer, websocket::Query::EvalGC(garbled_gates, constant_wires, garbler_input, evaluator_input, conversion_table))
        } else {
            panic!("Got a different response")
        }
        Ok(())
    }

    pub async fn create_garbler_input(&self, input : &BigUint, required_bits : u64) -> Vec<u8> {
        let garbler = self.garbler.lock().await;
        garbler.create_circuit_input(input, required_bits)
    }

    pub async fn create_evaluator_input(&self, input : &BigUint, required_bits : u64) -> (Vec<[PublicKey; 2]>, Vec<(SecretKey, u8)>) {
        let evaluator = self.evaluator.lock().await;
        evaluator.create_circuit_input(input, required_bits)
    }

    pub async fn get_preperation(&self) -> CircuitPrepeation {
        let preperation = self.preparation.lock().await;
        preperation.clone().expect("Preparation not set.")
    }

    pub fn get_peer_id(&self) -> PeerId {
        self.socket.get_peer_id()
    }

    pub fn get_address(&self) -> Multiaddr {
        self.socket.get_address()
    }

    async fn spawn_query_handler(self : Arc<Self>) {
        let mut incoming_streams = self.socket.get_control().accept(self.socket.get_protocol()).unwrap();
        tokio::spawn(async move {
            while let Some((peer, stream)) = incoming_streams.next().await {
                let handler_self = Arc::clone(&self);
                tokio::spawn(async move {
                    if let Err(e) = handler_self.handle_query(stream).await {
                        tracing::warn!(%peer, "Handle request error: {e}");
                    }
                });
            }
        });
    }

    // Req and reply
    async fn handle_query(&self, mut stream: Stream) -> Result<(), Box<dyn Error>> {
        let mut request_data = Vec::new();
        stream.read_to_end(&mut request_data).await?;
        
        let query: Query = postcard::from_bytes(&request_data).expect("Failed to deserialize the request");
        tracing::info!("Received request: {:?}", query);
        
        let response_enum = match query {
            Query::Hello => Response::Greeting("Hello".to_string()),
            Query::ExecuteProtocol => {

                Response::ProvideGC(garbled_gates, constant_wires, garbler_input, evaluator_input, conversion_table)
            }
        };

        let response_bytes = postcard::to_allocvec(&response_enum).expect("Failed to serialize response");

        stream.write_all(&response_bytes).await?;
        stream.close().await?;

        Ok(())
    }

}