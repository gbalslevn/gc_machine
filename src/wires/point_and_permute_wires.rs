use num_bigint::BigUint;
use rand::{thread_rng, Rng};
use crate::gates::gates::GateType;
use crate::wires::wires::Wires;
use crate::crypto_utils::generate_label_lsb;

pub struct PointAndPermuteWires {
    w0: BigUint,
    w1: BigUint
}

impl Wires for PointAndPermuteWires {
    fn w0(&self) -> &BigUint {
        &self.w0
    }

    fn w1(&self) -> &BigUint {
        &self.w1
    }

    fn generate_input_wire() -> Self {
        let mut rng = thread_rng();
        let choice = rng.gen_bool(1.0 / 2.0);
        let w0 = generate_label_lsb(choice);
        let w1 = generate_label_lsb(!choice);
        Self {w0: w0, w1: w1}
    }

    fn generate_output_wire(_wi: &Self, _wj: &Self, _gate: &GateType, _gate_id: &BigUint) -> Self {
        Self::generate_input_wire()
    }
}