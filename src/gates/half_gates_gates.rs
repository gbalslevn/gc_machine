use num_bigint::BigUint;
use crate::crypto_utils::gc_kdf_hg;
use crate::gates::gates::{Gate, GateType, Gates};
use crate::wires::wires::{Wire, Wires};

pub struct HalfGatesGates<W: Wires> {
    pub wires: W,
}

impl<W: Wires> Gates<W> for HalfGatesGates<W> {
    fn new(wires: W) -> Self {
        HalfGatesGates { wires }
    }

    fn generate_gate(&self, gate: GateType, wi: Wire, wj: Wire, gate_id: BigUint) -> Gate {
        let wo = self.wires.generate_output_wire(&wi, &wj, &gate, &gate_id);
        match gate {
            GateType::AND=> {
                let table = generate_and_table(&wi, &wj, &gate_id);
                Gate { gate_id, gate_type: GateType::AND, table, wi: wi, wj: wj, wo: wo }
            }
            GateType::XOR=> {
                Gate { gate_id, gate_type: GateType::XOR, table: Vec::new(), wi: wi, wj: wj, wo: wo }
            }
        }
    }
}

fn generate_and_table(wi: &Wire, wj: &Wire, gate_id: &BigUint) -> Vec<BigUint> {
    let mut table = vec![BigUint::from(0u8); 2];
    let tg;
    let te;
    let index_j;
    let index_j_prime;
    let delta = wi.w0() ^ wi.w1(); // could be given as argument through wiresGen struct
    tg = gc_kdf_hg(wi.w0(), index_j) ^ gc_kdf_hg(wi.w1(), index_j) ^
}

fn number_mult_bitstring(boolean: &bool, bitstring: &BigUint) -> BigUint {
    if (boolean) {
        bitstring.clone()
    } else {
        BigUint::from(0u8);
    }
}