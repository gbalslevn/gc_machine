use num_bigint::{BigUint};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::RngCore;
use crate::crypto_utils;
use crate::wires::wire_gen::{Wire, WireGen};

use crate::gates::gate_gen::{Gate, GateType, GateGen};
#[derive(Clone)]
pub struct OriginalGateGen<W: WireGen> {
    pub wire_gen: W,
    pub index: BigUint,
}

impl<W: WireGen> GateGen<W> for OriginalGateGen<W> {
    fn new(wire_gen: W) -> Self {
        OriginalGateGen { wire_gen, index: BigUint::from(0u32)}
    }

    fn generate_gate(&mut self, gate: GateType, wi: Wire, wj: Wire ) -> Gate {
        let wo = self.wire_gen.generate_output_wire(&wi, &wj, &gate, &self.index);
        let tt = self.get_tt(&wi, &wj, &wo, &gate);
        let mut table = vec![];
        // Creating symmetric key from left input, right input and gate id then encrypting the tt output with the key
        for (il, ir, out) in tt {
            let key = crypto_utils::gc_kdf(&il, &ir, &self.index);
            let zero_padded_out = out << 128;
            let ct = key ^ zero_padded_out;
            table.push(ct);
        }
        let mut rng = self.wire_gen.get_rng().clone();
        shuffle_vec(&mut rng, &mut table);
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

fn shuffle_vec(rng : &mut ChaCha20Rng, vec : &mut Vec<BigUint>) {
    // To get a uniformly distributed shuffle, once an element is set in its position, it should never be moved again. We therefore need to shuffle only 3 times as the 4th time, the element cannot be moved anyway 
    let len = vec.len();
    let mut byte = [0u8; 1];
    for i in 0..(len - 1)  {
        rng.fill_bytes(&mut byte);
        let choices_left = len - i;
        let random_index = byte[0] as usize % choices_left + i; // + i to avoid moving the same index twice. We go up the vec, such that index 0 is never swapped after iteration 0.
        vec.swap(i, random_index);
    }
}