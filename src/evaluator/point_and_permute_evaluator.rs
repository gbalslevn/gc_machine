use crate::evaluator::evaluator::Evaluator;
use crate::crypto_utils::gc_kdf_128;
use num_bigint::{BigUint};
pub struct PointAndPermuteEvaluator;

impl Evaluator for PointAndPermuteEvaluator {
    fn evaluate_and_gate(wi: &BigUint, wj: &BigUint, gate_id: &BigUint, gate: Vec<BigUint>) -> BigUint {
        let key = gc_kdf_128(wi, wj, gate_id);
        let pos = get_position(wi, wj);
        &gate[pos] ^ &key
    }

    // No difference between evaluation of AND gate and XOR gate
    fn evaluate_xor_gate(wi: &BigUint, wj: &BigUint, gate_id: &BigUint, gate: Vec<BigUint>) -> BigUint {
        Self::evaluate_and_gate(wi, wj, gate_id, gate)
    }
}

fn get_position(wi: &BigUint, wj: &BigUint) -> usize {
    let l = wi.bit(0) as usize;
    let r = wj.bit(0) as usize;
    l * 2 + r
}