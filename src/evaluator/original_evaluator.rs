use crate::{evaluator::evaluator::Evaluator, gates::gates::Gate};
use crate::crypto_utils::gc_kdf;
use num_bigint::{BigUint};
pub struct OriginalEvaluator;

impl Evaluator for OriginalEvaluator {
    fn evaluate_and_gate<W>(wi: &BigUint, wj: &BigUint, gate_id: &BigUint, gate: &Gate<W>) -> BigUint {
        let key = gc_kdf(wi, wj, gate_id);
        for entry in &gate.table {
            let dec = &key ^ entry;
            if dec.trailing_zeros().unwrap() >= 128 {
                return dec >> 128
            }
        }
        panic!("No output with correct padding found!");
    }

    // No difference between evaluation of AND gate and XOR gate
    fn evaluate_xor_gate<W>(wi: &BigUint, wj: &BigUint, gate_id: &BigUint, gate: &Gate<W>) -> BigUint {
        Self::evaluate_and_gate(wi, wj, gate_id, gate)
    }
}