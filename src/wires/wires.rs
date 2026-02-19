use num_bigint::{BigUint};
pub trait Wires {
    fn generate_input_wires(&self) -> (BigUint, BigUint);
    fn generate_output_wires(&self, wi: &(BigUint, BigUint), wj: &(BigUint, BigUint), gate: String, gate_id: &BigUint) -> (BigUint, BigUint);
}