use std::{collections::VecDeque, error::Error, sync::Arc};

use futures::{AsyncReadExt, AsyncWriteExt, StreamExt, lock::Mutex};
use k256::{PublicKey, SecretKey};
use libp2p::{Multiaddr, PeerId, Stream};
use num_bigint::BigUint;

use crate::{circuit_builder::CircuitBuild, evaluator::{evaluator::Evaluator}, garbler::Garbler, gates::gate_gen::GateGen, websocket::{self, Query, Response, SocketClient}, wires::wire_gen::WireGen};

// Peer can both be a garbler and a evaluator
pub struct Peer<G : GateGen<W>, W : WireGen, E : Evaluator> {
    garbler : Mutex<Garbler<G, W>>, // we use a mutex to enable mutability without having a mutable self
    evaluator : Mutex<E>,
    socket : SocketClient,
    context : Mutex<Option<CircuitContext>>  
}

#[derive(Clone)]
// Holds values needed to execute the protocol
pub struct CircuitContext {
    input : BigUint,
    build : CircuitBuild,
    required_bits : u64,
    evaluator_keys : Vec<(SecretKey, u8)>
}

impl <G : GateGen<W>, W : WireGen, E : Evaluator> Peer<G, W, E> where 
    G: GateGen<W> + Send + Sync + 'static,
    W: WireGen + Send + Sync + 'static,
    E: Evaluator + Send + Sync + 'static, {

    pub async fn new(garbler : Garbler<G, W>, evaluator : E) -> Arc<Self> {
        let socket = websocket::run().await.expect("Failed to start socket");
        
        let peer = Arc::new(Peer { garbler : garbler.into(), evaluator : evaluator.into(), socket, context : None.into() });
        // need to spawn a copy of the peer with Arc which handles ownership and enables to call self inside a new thread
        peer.clone().spawn_query_handler().await;
        
        peer
    }

    pub async fn setup_circuit_context(&self, input : BigUint, build : CircuitBuild, required_bits : u64) {
        let context = CircuitContext { input, build, required_bits, evaluator_keys : vec![] };
        let mut current_context = self.context.lock().await;
        *current_context = Some(context);
    } 

    pub async fn connect(&self, address : Multiaddr) -> Result<(), Box<dyn Error>> {
        self.socket.dial(address).await
    }

    // Garbler executes protocol
    pub async fn execute_protocol(&self, peer: PeerId) -> Result<Response, Box<dyn Error>> {
        // Get evuluators input
        let response = self.socket.send_query(peer, websocket::Query::ExecuteProtocol).await.expect("Error with query");
        if let Response::EvalInput(eval_input) = response {
            let circuit_preperation = self.get_circuit_context().await;
            let mut garbler = self.garbler.lock().await;
            let mut garbler_input =  garbler.create_circuit_input(&circuit_preperation.input, circuit_preperation.required_bits);
            let circuit = garbler.create_circuit(&circuit_preperation.build, &mut garbler_input, eval_input);

            // Get evaluator to evaluate circuit 
            let response = self.socket.send_query(peer, websocket::Query::EvaluateGC(circuit)).await;
            response
        } else {
            Err(format!("Protocol Violation: Expected EvalInput, got {:?}", response).into())
        }
    }

    pub async fn create_garbler_input(&self, input : &BigUint, required_bits : u64) -> VecDeque<u8> {
        let garbler = self.garbler.lock().await;
        garbler.create_circuit_input(input, required_bits)
    }

    pub async fn create_evaluator_input(&self, input : &BigUint, required_bits : u64) -> (VecDeque<[PublicKey; 2]>, Vec<(SecretKey, u8)>) {
        let evaluator = self.evaluator.lock().await;
        evaluator.create_circuit_input(input, required_bits)
    }

    pub async fn get_circuit_context(&self) -> CircuitContext {
        let context = self.context.lock().await;
        context.clone().expect("Circuit context not set")
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

    async fn insert_secret_keys(&self, evaluator_keys : Vec<(SecretKey, u8)>) {
        let mut current_context = self.context.lock().await;
        let context_as_mut = current_context.as_mut().expect("Could not get context");
        context_as_mut.evaluator_keys = evaluator_keys;
    } 

    // Empties context
    async fn reset_circuit_context(&self) {
        let mut current_context = self.context.lock().await;
        *current_context = None.into()
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
                let context = self.get_circuit_context().await;
                let evaluator = self.evaluator.lock().await;
                let (eval_input, secret_keys) = evaluator.create_circuit_input(&context.input, context.required_bits);
                self.insert_secret_keys(secret_keys).await;
                Response::EvalInput(eval_input)
            }
            Query::EvaluateGC(circuit) => {
                let mut evaluator = self.evaluator.lock().await;
                let context = self.get_circuit_context().await;

                let result = evaluator.evaluate_circuit(&context.build, &circuit.gates, &circuit.constant_wires, &circuit.garbler_input, &circuit.evaluator_input, context.evaluator_keys, circuit.output_conversion);
                self.reset_circuit_context().await;
                Response::GCResult(result)
            }
        };

        let response_bytes = postcard::to_allocvec(&response_enum).expect("Failed to serialize response");

        stream.write_all(&response_bytes).await?;
        stream.close().await?;

        Ok(())
    }

}