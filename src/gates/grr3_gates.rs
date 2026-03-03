use num_bigint::BigUint;
use crate::crypto_utils;
use crate::gates::gates::{Gate, GateType, Gates};
use crate::wires::wires::{Wire, Wires};

pub struct GRR3Gates<W: Wires> {
    pub wires: W,
    pub index: BigUint,
}

impl<W: Wires> Gates<W> for GRR3Gates<W>  {
    fn new(wires: W) -> Self {
        GRR3Gates{ wires, index: BigUint::from(0u32), }
    }

    fn generate_gate(&mut self, gate: GateType, wi: Wire, wj: Wire) -> Gate {
        let wo = self.wires.generate_output_wire(&wi, &wj, &gate, &self.index);
        let tt = self.get_tt(&wi, &wj, &wo, &gate);
        let mut table = vec![BigUint::from(0u8); 3];
        // Creating symmetric key from left input, right input and gate id then encrypting the tt output with the key
        for (il, ir, out) in tt {
            let key = crypto_utils::gc_kdf_128(&il, &ir, &self.index);
            let ct = key ^ out;
            let pos = get_position(&il, &ir);
            if pos != 0 {
                table[pos-1] = ct;
            }
        }
        let gate = Gate {
            gate_type: gate, table, wi, wj, wo
        };
        self.increment_index();
        gate
    }
    fn get_index(&self) -> &BigUint {
        &self.index
    }

    fn increment_index(&mut self) -> &BigUint {
        self.index += 1u32;
        &self.index
    }
}

pub fn get_position(il: &BigUint, ir: &BigUint) -> usize {
    let l = il.bit(0) as usize;
    let r = ir.bit(0) as usize;
    l * 2 + r
}