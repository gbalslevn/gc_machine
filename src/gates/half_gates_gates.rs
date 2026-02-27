use num_bigint::BigUint;
use crate::crypto_utils::gc_kdf_hg;
use crate::gates::gates::{Gate, GateType, Gates};
use crate::wires::half_gates_wires::HalfGateWires;
use crate::wires::wires::{Wire, Wires};

pub struct HalfGatesGates{
    pub wires: HalfGateWires,
    index: BigUint,

}

impl Gates<HalfGateWires> for HalfGatesGates {
    fn new(wires: HalfGateWires) -> Self {
        HalfGatesGates { wires, index: BigUint::from(0u32)}
    }

    fn generate_gate(&mut self, gate: GateType, wi: Wire, wj: Wire) -> Gate {
        let wo = self.wires.generate_output_wire(&wi, &wj, &gate, &self.index);
        match gate {
            GateType::AND=> {
                let table = generate_and_table(&wi, &wj, &self.index);
                let gate = Gate { gate_type: GateType::AND, table, wi, wj, wo };
                self.increment_index();
                self.increment_index();
                gate
            }
            GateType::XOR=> {
                Gate { gate_type: GateType::XOR, table: Vec::new(), wi, wj, wo }
            }
        }
    }

    fn get_index(&self) -> &BigUint {
        &self.index
    }

    fn increment_index(&mut self) -> &BigUint{
        self.index += 1u32;
        &self.index
    }
}


fn generate_and_table(wi: &Wire, wj: &Wire, index: &BigUint) -> Vec<BigUint> {
    let tg;
    let te;
    let delta = wi.w0() ^ wi.w1(); // could be given as argument through wiresGen struct
    let index_j = index;
    let pb_r = bit_mult_number(&wj.w0().bit(0), &delta);
    tg = gc_kdf_hg(wi.w0(), index_j) ^ gc_kdf_hg(wi.w1(), index_j) ^ pb_r;

    let index_j_prime = index_j + 1u32;
    te = gc_kdf_hg(wj.w0(), &index_j_prime) ^ gc_kdf_hg(wj.w1(), &index_j_prime) ^ wi.w0();
    vec!(tg, te)
}
fn bit_mult_number(boolean: &bool, bitstring: &BigUint) -> BigUint {
    if *boolean {
        bitstring.clone()
    } else {
        BigUint::from_bytes_be(&[0u8; 16])
    }
}