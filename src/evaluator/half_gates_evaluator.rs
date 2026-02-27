use crate::evaluator::evaluator::Evaluator;
use crate::crypto_utils::gc_kdf_hg;
use num_bigint::{BigUint};

pub struct HalfGatesEvaluator {
    index: BigUint,
}

impl HalfGatesEvaluator {
    pub fn new() -> Self {
        HalfGatesEvaluator {
            index: BigUint::from(0u32),
        }
    }
}

impl Evaluator for HalfGatesEvaluator {
    fn evaluate_and_gate(&mut self, wi: &BigUint, wj: &BigUint, table: &Vec<BigUint>) -> BigUint {
        let sa = wi.bit(0);
        let wg = garbler_half_gate(sa, wi, self.get_index(), &table[0]);
        self.increment_index();

        let sb = wj.bit(0);
        let we = evaluator_half_gate(sb, wi, wj, self.get_index(), &table[1]);
        self.increment_index();

        wg ^ we
    }

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

fn garbler_half_gate(sa: bool, wi: &BigUint, index: &BigUint, tg: &BigUint) -> BigUint {
    if sa {
        gc_kdf_hg(wi, index) ^ tg
    } else {
        gc_kdf_hg(wi, index)
    }
}

fn evaluator_half_gate(sb: bool, wi: &BigUint, wj: &BigUint, index: &BigUint, te: &BigUint) -> BigUint {
    if sb {
        gc_kdf_hg(wj, index) ^ te ^ wi
    } else {
        gc_kdf_hg(wj, index)
    }
}