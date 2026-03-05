use num_bigint::BigUint;
use rand_chacha::ChaCha20Rng;
use crate::{crypto_utils, gates::gates::GateType, wires::wires::{Wire, Wires}};
use rand_chacha::rand_core::{RngCore};

#[derive(Clone)]
pub struct OriginalWires {
    rng: ChaCha20Rng
}

impl Wires for OriginalWires {
    fn new() -> Self { 
        let rng = crypto_utils::gen_rng();
        Self { rng }
    }
    fn generate_input_wire(&mut self) -> Wire {
        generate_wire(&mut self.rng)
    }

    fn generate_output_wire(&mut self, _wi: &Wire, _wj: &Wire, _gate: &GateType, _gate_id: &BigUint) -> Wire {
        generate_wire(&mut self.rng)
    }
    fn get_rng(&self) -> &ChaCha20Rng {
        &self.rng
    }
    fn new_rng(&mut self) {
        self.rng = crypto_utils::gen_rng();
    }
}

fn generate_wire(rng : &mut ChaCha20Rng) -> Wire {
    let mut bytes0 = [0u8; 16]; // 128 bits
        let mut bytes1 = [0u8; 16]; // 128 bits

        rng.fill_bytes(&mut bytes0);
        rng.fill_bytes(&mut bytes1);

        let w0 = BigUint::from_bytes_be(&bytes0);
        let w1 = BigUint::from_bytes_be(&bytes1);
        Wire::new(w0, w1)
}