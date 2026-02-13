use num_bigint::BigUint;
use rand::{thread_rng, Rng};
use crate::wires::wires::Wires;

pub struct OriginalWires;

impl Wires for OriginalWires {
    fn generate_input_wires() -> (BigUint, BigUint) {
        let mut bytes0 = [0u8; 16]; // 128 bits
        let mut bytes1 = [0u8; 16]; // 128 bits

        thread_rng().fill(&mut bytes0);
        thread_rng().fill(&mut bytes1);

        (BigUint::from_bytes_be(&bytes0), BigUint::from_bytes_be(&bytes1))
    }

    fn generate_output_wires(_w0i: &BigUint, _w1i: &BigUint, _w0j: &BigUint, _w1j: &BigUint, _gate: String, _gate_id: &BigUint) -> (BigUint, BigUint) {
        Self::generate_input_wires()
    }
}