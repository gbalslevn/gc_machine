use num_bigint::BigUint;
use crate::crypto_utils;
use crate::gates::gates::{Gate, GateType, Gates};
use crate::wires::wires::{Wire, Wires};

pub struct PointAndPermuteGates<W: Wires> {
    pub wires: W,
}

impl<W: Wires> Gates<W> for PointAndPermuteGates<W> {
    fn new(wires: W) -> Self {
        PointAndPermuteGates { wires }
    }

    fn generate_gate(&self, gate: GateType, wi: Wire, wj: Wire, gate_id: BigUint) -> Gate {
        let wo = self.wires.generate_output_wire(&wi, &wj, &gate, &gate_id);
        let tt = self.get_tt(&wi, &wj, &wo, &gate);
        let mut table = vec![BigUint::from(0u8); 4];
        // Creating symmetric key from left input, right input and gate id then encrypting the tt output with the key
        for (il, ir, out) in tt {
            let key = crypto_utils::gc_kdf_128(&il, &ir, &gate_id);
            let ct = key ^ out;
            let pos = get_position(&il, &ir);
            table[pos]= ct;
        }
        Gate {
            gate_id: gate_id, gate_type: gate, table: table, wi : wi, wj: wj, wo: wo
        }
    }

}

pub fn get_position(il: &BigUint, ir: &BigUint) -> usize {
    let l = il.bit(0) as usize;
    let r = ir.bit(0) as usize;
    l * 2 + r
}