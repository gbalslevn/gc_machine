use num_bigint::BigUint;
use crate::crypto_utils;
use crate::gates::gates::{Gate, GateType, Gates};
use crate::wires::wires::{Wire, Wires};
pub struct FreeXORGates<W: Wires> {
    pub wires: W,
    pub index: BigUint,
}

// Implements free XOR and grr3

impl<W: Wires> Gates<W> for FreeXORGates<W> {
    fn new(wires: W) -> Self {
        FreeXORGates { wires, index: BigUint::from(0u32), }
    }
    fn generate_gate(&mut self, gate: GateType, wi: Wire, wj: Wire) -> Gate {
        let wo = self.wires.generate_output_wire(&wi, &wj, &gate, &self.index);
        let tt = self.get_tt(&wi, &wj, &wo, &gate);
        match gate {
            GateType::AND=> {
                let table = generate_and_table(&tt,  &self.index);
                let gate = Gate { gate_type: GateType::AND, table, wi, wj, wo };
                self.increment_index();
                gate
            }
            GateType::XOR=>Gate { gate_type: GateType::XOR, table: Vec::new(), wi, wj, wo }
        }
    }

    fn get_index(&self) -> &BigUint {
        &self.index
    }

    fn increment_index(&mut self) -> &BigUint {
        self.index += 1u32;
        &self.index
    }
}

fn generate_and_table(tt : &[(BigUint, BigUint, BigUint); 4], gate_id: &BigUint) -> Vec<BigUint> {
    let mut table = vec![BigUint::from(0u8); 3];
    // Creating symmetric key from left input, right input and gate id then encrypting the tt output with the key
    for (il, ir, out) in tt {
        let key = crypto_utils::gc_kdf_128(il, ir, gate_id);
        let ct = key ^ out;
        let pos = crate::gates::grr3_gates::get_position(il, ir);
        if pos != 0 {
            table[pos-1] = ct;
        }
    }
    table
}


pub fn get_position(il: &BigUint, ir: &BigUint) -> usize {
    let l = il.bit(0) as usize;
    let r = ir.bit(0) as usize;
    l * 2 + r
}