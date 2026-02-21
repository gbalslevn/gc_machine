use num_bigint::BigUint;
use crate::crypto_utils;
use crate::wires::original_wires::OriginalWires;
use crate::wires::wires::Wires;

use rand::{thread_rng};
use rand::seq::SliceRandom;
use crate::gates::gates::{Gate, GateType, Gates};
pub struct OriginalGates;

impl Gates<OriginalWires> for OriginalGates {
    fn new(gate : &GateType, gate_id: BigUint) -> Gate<OriginalWires> {
        let wi = OriginalWires::generate_input_wire();
        let wj = OriginalWires::generate_input_wire();
        let wo = OriginalWires::generate_output_wire(&wi, &wj, gate, &gate_id);
        let tt = OriginalGates.get_tt(&wi, &wj, &wo, &gate);
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
            gate_id: gate_id, table: table, wi : wi, wj: wj, wo: wo
        }
    }
}

