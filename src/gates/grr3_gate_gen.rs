use num_bigint::BigUint;
use crate::crypto_utils;
use crate::crypto_utils::gen_rng_with_seed;
use crate::gates::gate_gen::{Gate, GateType, GateGen};
use crate::wires::grr3_wire_gen::GRR3WireGen;
use crate::wires::wire_gen::{Wire, WireGen};

pub struct GRR3GateGen {
    wire_gen: GRR3WireGen,
    index: BigUint,
}

impl GateGen for GRR3GateGen  {
    type W = GRR3WireGen;
    fn new() -> Self {
        let wire_gen = GRR3WireGen::new();
        GRR3GateGen { wire_gen, index: BigUint::from(0u32), }
    }

    fn new_with_seed(seed: &BigUint) -> Self {
        let rng = gen_rng_with_seed(seed);
        let wire_gen = GRR3WireGen::new_with_rng(rng);
        GRR3GateGen { wire_gen, index: BigUint::from(0u32) }
    }
    fn generate_gate(&mut self, gate: GateType, wi: Wire, wj: Wire) -> Gate {
        let wo = self.wire_gen.generate_output_wire(&wi, &wj, &gate, &self.index);
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

    fn get_wire_gen(&mut self) -> &mut Self::W {
        &mut self.wire_gen
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