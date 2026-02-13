use num_bigint::{BigUint};
pub trait Wires {
    fn generate_input_wires() -> (BigUint, BigUint);
    fn generate_output_wires(w0i: &BigUint, w1i: &BigUint, w0j: &BigUint, w1j: &BigUint, gate: String, gate_id: &BigUint) -> (BigUint, BigUint);
}