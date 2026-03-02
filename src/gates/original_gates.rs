use num_bigint::BigUint;
use crate::crypto_utils;
use crate::wires::wires::{Wire, Wires};

use rand::{thread_rng};
use rand::seq::SliceRandom;
use crate::gates::gates::{Gate, GateType, Gates};
pub struct OriginalGates<W: Wires> {
    pub wires: W,
}

impl<W: Wires> Gates<W> for OriginalGates<W> {
    fn new(wires: W) -> Self {
        OriginalGates{wires}
    }

    fn generate_gate(&self, gate: GateType, wi: Wire, wj: Wire, gate_id: BigUint) -> Gate {
        let wo = self.wires.generate_output_wire(&wi, &wj, &gate, &gate_id);
        let tt = self.get_tt(&wi, &wj, &wo, &gate);
        let mut table = vec![];
        // Creating symmetric key from left input, right input and gate id then encrypting the tt output with the key
        for (il, ir, out) in tt {
            let key = crypto_utils::gc_kdf(&il, &ir, &gate_id);
            let zero_padded_out = out << 128;
            let ct = key ^ zero_padded_out;
            table.push(ct);
        }
        table.shuffle(&mut thread_rng());
        Gate {
            gate_id: gate_id, gate_type: gate, table: table, wi : wi, wj: wj, wo: wo
        }
    }
}