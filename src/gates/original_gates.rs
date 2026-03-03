use num_bigint::BigUint;
use crate::crypto_utils;
use crate::wires::wires::{Wire, Wires};

use rand::{thread_rng};
use rand::seq::SliceRandom;
use crate::gates::gates::{Gate, GateType, Gates};
pub struct OriginalGates<W: Wires> {
    pub wires: W,
    pub index: BigUint,
}

impl<W: Wires> Gates<W> for OriginalGates<W> {
    fn new(wires: W) -> Self {
        OriginalGates{ wires, index: BigUint::from(0u32)}
    }

    fn generate_gate(&mut self, gate: GateType, wi: Wire, wj: Wire ) -> Gate {
        let wo = self.wires.generate_output_wire(&wi, &wj, &gate, &self.index);
        let tt = self.get_tt(&wi, &wj, &wo, &gate);
        let mut table = vec![];
        // Creating symmetric key from left input, right input and gate id then encrypting the tt output with the key
        for (il, ir, out) in tt {
            let key = crypto_utils::gc_kdf(&il, &ir, &self.index);
            let zero_padded_out = out << 128;
            let ct = key ^ zero_padded_out;
            table.push(ct);
        }
        table.shuffle(&mut thread_rng());
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