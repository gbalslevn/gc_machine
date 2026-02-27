use crate::evaluator::evaluator::Evaluator;
use crate::crypto_utils::gc_kdf;
use num_bigint::BigUint;
pub struct OriginalEvaluator;

impl Evaluator for OriginalEvaluator {
    fn evaluate_and_gate(wi: &BigUint, wj: &BigUint, gate_id: &BigUint, table: &Vec<BigUint>) -> BigUint {
        let key = gc_kdf(wi, wj, gate_id);
        for entry in table {
            let dec = &key ^ entry;
            println!("dec label is: {}", dec);
            if dec.trailing_zeros().unwrap() >= 128 {
                println!("label has trailing zeros: {}", dec);
                return dec >> 128
            }
        }
        panic!("No decryption with correct padding found!");
    }

    // No difference between evaluation of AND gate and XOR gate
    fn evaluate_xor_gate(wi: &BigUint, wj: &BigUint, gate_id: &BigUint, table: &Vec<BigUint>) -> BigUint {
        Self::evaluate_and_gate(wi, wj, gate_id, table)
    }
}