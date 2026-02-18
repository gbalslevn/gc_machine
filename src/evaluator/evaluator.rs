use num_bigint::{BigUint};
pub trait Evaluator {
    fn evaluate_gate(wi: &BigUint, wj: &BigUint, gate_id: &BigUint, gate_type: String, gate: &Vec<BigUint>) -> BigUint {
        match gate_type.as_str() {
            "and" => Self::evaluate_and_gate(wi, wj, gate_id, gate),
            "xor" => Self::evaluate_xor_gate(wi, wj, gate_id, gate),
            _ => panic!("Unknown gate {}", gate_type),
        }
    }

    fn evaluate_and_gate(key: &BigUint, wj: &BigUint, gate_id: &BigUint, gate: &Vec<BigUint>) -> BigUint;
    fn evaluate_xor_gate(wi: &BigUint, wj: &BigUint, gate_id: &BigUint, gate: &Vec<BigUint>) -> BigUint;
}