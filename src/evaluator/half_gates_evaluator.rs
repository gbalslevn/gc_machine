use crate::{evaluator::evaluator::Evaluator};
use crate::crypto_utils::{gc_kdf_hg, gc_kdf, gc_kdf_mux, gen_rng, gen_rng_with_seed};
use num_bigint::{BigUint, ToBigUint};
use crate::circuit_builder::CircuitBuild;
use crate::garbler::Circuit;
use crate::gates::gate_gen::GateGen;
use crate::gates::half_gates_gate_gen::HalfGatesGateGen;
use crate::wires::half_gates_wire_gen::HalfGatesWireGen;
use crate::wires::wire_gen::WireGen;

pub struct HalfGatesEvaluator {
    index: BigUint,
}

impl HalfGatesEvaluator {
    pub fn new() -> Self {
        HalfGatesEvaluator {
            index: BigUint::from(0u32),
        }
    }

    pub fn evaluate_demux(&mut self, w: &BigUint, seed: &BigUint, demux: &Vec<BigUint>) -> (BigUint, BigUint) {
        let pos = get_position(w, seed);
        let key = gc_kdf(w, seed, self.get_index());
        self.increment_index();
        let output = key ^ &demux[pos];
        let output_bytes = output.to_bytes_be();
        let if_wire  = BigUint::from_bytes_be(&output_bytes[..16]);  // first 128 bits
        let else_wire = BigUint::from_bytes_be(&output_bytes[16..]);  // last 128 bits
        (if_wire, else_wire)
    }

    pub fn unstack_material(&mut self, seed: &BigUint, m_cond: &Vec<Vec<BigUint>>, circuit_build: CircuitBuild) -> Vec<Vec<BigUint>> {
        todo!()
    }

    pub fn generate_subcircuit(seed: &BigUint, circuit_build: CircuitBuild) -> Vec<Vec<BigUint>> {
        // let index = 0.to_biguint().unwrap();
        // let mut gate_gen = HalfGatesGateGen::new_with_seed(seed);
        todo!()
    }

    pub fn evaluate_mux(&mut self, wi: &BigUint, wj: &BigUint, seed: &BigUint, mux: &Vec<BigUint>) -> BigUint {
        let pos = get_position(wi, wj);
        let key = gc_kdf_mux(seed, wi, wj, self.get_index());
        self.increment_index();
        key ^ &mux[pos]
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
    let mut ge = gc_kdf_hg(wj, index);
    if sb {
        ge = ge ^ te ^ wi
    }
    ge
}

fn get_position(wi: &BigUint, wj: &BigUint) -> usize {
    let l = wi.bit(0) as usize;
    let r = wj.bit(0) as usize;
    l * 2 + r
}