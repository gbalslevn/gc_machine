use num_bigint::{BigUint};
use rand_chacha::ChaCha20Rng;

use crate::gates::gate_gen::GateType;

// A wire has two label values, representing bit 0 and 1. 
pub trait WireGen {
    fn new() -> Self;
    fn generate_input_wire(&mut self) -> Wire;  
    fn generate_output_wire(&mut self, wi: &Wire, wj: &Wire, gate: &GateType, index: &BigUint) -> Wire;
    fn get_rng(&self) -> &ChaCha20Rng;
    fn new_rng(&mut self);
}

#[derive(Debug, Clone, PartialEq)]
pub struct Wire {
    w0: BigUint,
    w1: BigUint,
}

impl Wire {
    pub fn new(w0: BigUint, w1: BigUint) -> Self {
        Wire {w0, w1}
    }
    pub fn w0(&self) -> &BigUint {
        &self.w0
    }

    pub fn w1(&self) -> &BigUint {
        &self.w1
    }
}
