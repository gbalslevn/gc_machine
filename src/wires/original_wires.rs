use num_bigint::BigUint;
use rand::{thread_rng, Rng};
use crate::{gates::gates::GateType, wires::wires::Wires};

pub struct OriginalWires {
    pub w0: BigUint,
    pub w1: BigUint
}

impl Wires for OriginalWires {
    fn w0(&self) -> &BigUint {
        &self.w0
    }

    fn w1(&self) -> &BigUint {
        &self.w1
    }

    fn generate_input_wire() -> OriginalWires {
        let mut bytes0 = [0u8; 16]; // 128 bits
        let mut bytes1 = [0u8; 16]; // 128 bits

        thread_rng().fill(&mut bytes0);
        thread_rng().fill(&mut bytes1);

        let w0 = BigUint::from_bytes_be(&bytes0);
        let w1 = BigUint::from_bytes_be(&bytes1);
        Self { w0 , w1 }
    }

    fn generate_output_wire(_wi: &Self, _wj: &Self, _gate: &GateType, _gate_id: &BigUint) -> Self {
        Self::generate_input_wire()
    }
}