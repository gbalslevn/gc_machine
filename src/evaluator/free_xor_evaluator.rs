use crate::evaluator::evaluator::Evaluator;
use crate::crypto_utils::gc_kdf_128;
use num_bigint::{BigUint};
pub struct FreeXOREvaluator {
    index: BigUint,
}

impl FreeXOREvaluator {
    pub fn new() -> Self {
        FreeXOREvaluator {
            index: BigUint::from(0u32),
        }
    }
}
impl Evaluator for FreeXOREvaluator {
    fn evaluate_and_gate(&mut self, wi: &BigUint, wj: &BigUint, table: &Vec<BigUint>) -> BigUint {
        let key = gc_kdf_128(wi, wj, self.get_index()); // Evaluator needs a gatetype, two bits and the table
        self.increment_index();
        let pos = get_position(wi, wj);
        if pos == 0 {
            key.clone()
        } else {
            &table[pos-1] ^ &key
        }
    }

    // No difference between evaluation of AND gate and XOR gate
    fn evaluate_xor_gate(&mut self, wi: &BigUint, wj: &BigUint, _table: &Vec<BigUint>) -> BigUint {
        wi ^ wj
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