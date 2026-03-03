use crate::evaluator::evaluator::Evaluator;
use crate::crypto_utils::gc_kdf;
use num_bigint::BigUint;
pub struct OriginalEvaluator {
    index: BigUint,
}

impl OriginalEvaluator {
    pub fn new() -> Self {
        OriginalEvaluator {
            index: BigUint::from(0u32),
        }
    }
}

impl Evaluator for OriginalEvaluator {
    fn evaluate_and_gate(&mut self, wi: &BigUint, wj: &BigUint, table: &Vec<BigUint>) -> BigUint {
        let key = gc_kdf(wi, wj, self.get_index());
        self.increment_index();
        for entry in table {
            let dec = &key ^ entry;
            if dec.trailing_zeros().unwrap() >= 128 {
                return dec >> 128
            }
        }
        panic!("No decryption with correct padding found!");
    }

    // No difference between evaluation of AND gate and XOR gate
    fn evaluate_xor_gate(&mut self, wi: &BigUint, wj: &BigUint, table: &Vec<BigUint>) -> BigUint {
        self.evaluate_and_gate(wi, wj, table)
    }

    fn increment_index(&mut self) {
        self.index += 1u32;
    }

    fn get_index(&self) -> &BigUint {
        &self.index
    }
}