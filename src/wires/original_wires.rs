use num_bigint::BigUint;
use rand::{thread_rng, Rng};
use crate::{gates::gates::GateType, wires::wires::{Wire, Wires}};

pub struct OriginalWires;

impl Wires for OriginalWires {
    fn new() -> Self {
        Self
    }
    fn generate_input_wire(&self) -> Wire {
        generate_wire()
    }

    fn generate_output_wire(_wi: &Wire, _wj: &Wire, _gate: &GateType, _gate_id: &BigUint) -> Wire {
        generate_wire()
    }
}

fn generate_wire() -> Wire {
    let mut bytes0 = [0u8; 16]; // 128 bits
        let mut bytes1 = [0u8; 16]; // 128 bits

        thread_rng().fill(&mut bytes0);
        thread_rng().fill(&mut bytes1);

        let w0 = BigUint::from_bytes_be(&bytes0);
        let w1 = BigUint::from_bytes_be(&bytes1);
        Wire::new(w0, w1)
}