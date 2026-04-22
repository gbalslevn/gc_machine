use std::collections::HashMap;
use crate::{evaluator::evaluator::Evaluator};
use crate::crypto_utils::{gc_kdf_hg, gc_kdf, gc_kdf_mux, gen_rng, gen_rng_with_seed};
use num_bigint::{BigUint, ToBigUint};
use crate::circuit_builder::{CircuitBuild, SubcircuitBuild};
use crate::garbler::Circuit;
use crate::gates::gate_gen::GateGen;
use crate::gates::half_gates_gate_gen::HalfGatesGateGen;
use crate::wires::half_gates_wire_gen::HalfGatesWireGen;
use crate::wires::wire_gen::{Wire, WireGen};

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

    pub fn unstack_material(&mut self, seed: &BigUint, m_cond: &Vec<Vec<BigUint>>, subcircuit_build: SubcircuitBuild) -> Vec<Vec<BigUint>> {
        let material = Self::generate_subcircuit(seed, subcircuit_build);

        // This function makes an XOR between each garbled entry in the stacked material m_cond and the generated subcircuit
        let unstacked_material: Vec<Vec<BigUint>> = m_cond
            .iter()
            .zip(material.iter())
            .map(|(cond_row, mat_row)| {
                cond_row
                    .iter()
                    .zip(mat_row.iter())
                    .map(|(cond_val, mat_val)| cond_val ^ mat_val)
                    .collect()
            })
            .collect();
        unstacked_material
    }

    pub fn generate_subcircuit(seed: &BigUint, subcircuit_build: SubcircuitBuild) -> Vec<Vec<BigUint>> {
        let mut gate_gen = HalfGatesGateGen::new_with_seed(seed);

        let mut known_wires: HashMap<BigUint, Wire> = HashMap::new();
        let input_wire = subcircuit_build.input_wires;
        let wire = gate_gen.get_wire_gen().generate_input_wire();
        known_wires.insert(input_wire.wire_id().clone(), wire);
        // for wirebuild in input_wires {
        //     let wire = gate_gen.get_wire_gen().generate_input_wire();
        //     known_wires.insert(wirebuild.wire_id().clone(), wire.clone());
        // }

        let gates = subcircuit_build.gates;
        let mut subcircuit: Vec<Vec<BigUint>> = Vec::new();
        for gate in gates {
            let wi = known_wires.get(&gate.wi().wire_id()).unwrap().clone();
            let wj = known_wires.get(&gate.wj().wire_id()).unwrap().clone();

            let new_gate = gate_gen.generate_gate(
                gate.gate_type().clone(),
                wi.clone(),
                wj.clone()
            );

            let output_wire_id = gate.wo().wire_id();
            known_wires.insert(output_wire_id.clone(), new_gate.wo.clone());
            let table = new_gate.to_table();

            // Store the ciphertexts for the gate
            subcircuit.push(table);
        }
        subcircuit
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