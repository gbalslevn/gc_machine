use crate::evaluator::evaluator::Evaluator;
use crate::crypto_utils::gc_kdf_128;
use num_bigint::{BigUint};
pub struct PointAndPermuteEvaluator {
    index: BigUint,
}

impl PointAndPermuteEvaluator {
    pub fn new() -> Self {
        PointAndPermuteEvaluator {
            index: BigUint::from(0u32),
        }
    }
}

impl Evaluator for PointAndPermuteEvaluator {
    fn evaluate_and_gate(&mut self, wi: &BigUint, wj: &BigUint, table: &Vec<BigUint>) -> BigUint {
        let key = gc_kdf_128(wi, wj, self.get_index());
        self.increment_index();
        let pos = get_position(wi, wj);
        &table[pos] ^ &key
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

fn get_position(wi: &BigUint, wj: &BigUint) -> usize {
    let l = wi.bit(0) as usize;
    let r = wj.bit(0) as usize;
    l * 2 + r
}