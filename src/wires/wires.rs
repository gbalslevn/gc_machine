use num_bigint::{BigUint};

use crate::gates::gates::GateType;

// A wire has two label values, representing bit 0 and 1. 
pub trait Wires {
    fn generate_input_wire() -> Self;
    fn generate_output_wire(wi: &Self, wj: &Self, gate: &GateType, gate_id: &BigUint) -> Self; 
    fn new(w0: BigUint, w1: BigUint) -> Self;
    fn w0(&self) -> &BigUint;
    fn w1(&self) -> &BigUint;
}

impl Wire {
    pub fn new(w0: BigUint, w1: BigUint) -> Self {
        Self { w0, w1 }
    }
    pub fn w0(&self) -> &BigUint {
        &self.w0
    }

    pub fn w1(&self) -> &BigUint {
        &self.w1
    }
}

pub struct Wire {
    w0: BigUint,
    w1: BigUint
}