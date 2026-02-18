use num_bigint::BigUint;
use rand::{thread_rng, Rng};
use crate::wires::wires::Wires;
use crate::crypto_utils::generate_label_lsb;

pub struct PointAndPermuteWires;

impl Wires for PointAndPermuteWires {
    fn generate_input_wires() -> (BigUint, BigUint) {
        let mut rng = thread_rng();
        let choice = rng.gen_bool(1.0 / 2.0);
        let w0 = generate_label_lsb(choice);
        let w1 = generate_label_lsb(!choice);
        (w0, w1)
    }

    fn generate_output_wires(_wi: &(BigUint, BigUint), _wj: &(BigUint, BigUint), _gate: String, _gate_id: &BigUint) -> (BigUint, BigUint) {
        Self::generate_input_wires()
    }
}