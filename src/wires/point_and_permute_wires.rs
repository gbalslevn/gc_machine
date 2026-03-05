use num_bigint::BigUint;
use rand_chacha::ChaCha20Rng;
use crate::gates::gates::GateType;
use crate::wires::wires::{Wire, Wires};
use crate::crypto_utils::{self, generate_label_lsb};

#[derive(Clone)]
pub struct PointAndPermuteWires {
    rng : ChaCha20Rng
}

impl Wires for PointAndPermuteWires {
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
        self.rng = crypto_utils::gen_rng()
    }
}

fn generate_wire(rng : &mut ChaCha20Rng) -> Wire {
        let choice = crypto_utils::gen_bool(rng);
        let w0 = generate_label_lsb(rng, choice);
        let w1 = generate_label_lsb(rng, !choice);
        Wire::new(w0, w1)
}