use std::ops::Add;

use crate::{evaluator::evaluator::Evaluator};
use crate::crypto_utils::{self, gc_kdf_128};
use num_bigint::{BigUint};
pub struct GRR3Evaluator {
    index: BigUint,
}

impl GRR3Evaluator {
    pub fn new() -> Self {
        GRR3Evaluator {
            index: BigUint::from(0u32),
        }
    }
}

impl Evaluator for GRR3Evaluator {
    fn evaluate_and_gate(&mut self, wi: &BigUint, wj: &BigUint, table: &Vec<BigUint>) -> BigUint {
        let index = self.get_index().clone();
        let key = gc_kdf_128(wi, wj, &index);
        let pos = get_position(wi, wj);
        self.increment_index();
        if pos == 0 {
            let mn = crypto_utils::get_magic_number();
            gc_kdf_128(&wi.add(mn), wj, &index)
        } else {
            &table[pos-1] ^ &key
        }
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