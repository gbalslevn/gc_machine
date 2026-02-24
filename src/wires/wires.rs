use num_bigint::{BigUint};

use crate::gates::gates::GateType;

// A wire has two label values, representing bit 0 and 1. 
pub trait Wires {
    fn new() -> Self;
    fn generate_input_wire(&self) -> Wire;
    fn generate_output_wire(wi: &Wire, wj: &Wire, gate: &GateType, gate_id: &BigUint) -> Wire; 
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

#[derive(Debug, Clone)]
pub struct Wire {
    w0: BigUint,
    w1: BigUint
}