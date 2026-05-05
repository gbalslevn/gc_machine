use num_bigint::BigUint;
use crate::crypto_utils;
use crate::crypto_utils::gen_rng_with_seed;
use crate::gates::gate_gen::{Gate, GateType, GateGen};
use crate::wires::free_xor_wire_gen::FreeXORWireGen;
use crate::wires::wire_gen::{Wire, WireGen};
pub struct FreeXORGateGen {
    wire_gen: FreeXORWireGen,
    index: BigUint,
}

// Implements free XOR and grr3
impl GateGen for FreeXORGateGen {
    type W = FreeXORWireGen;
    fn new() -> Self {
        let wire_gen = FreeXORWireGen::new();
        FreeXORGateGen { wire_gen, index: BigUint::from(0u32), }
    }

    fn new_with_seed(seed: &BigUint) -> Self {
        let rng = gen_rng_with_seed(seed);
        let wire_gen = FreeXORWireGen::new_with_rng(rng);
        FreeXORGateGen { wire_gen, index: BigUint::from(0u32) }
    }
    
    fn generate_gate(&mut self, gate: GateType, wi: Wire, wj: Wire) -> Gate {
        let wo = self.wire_gen.generate_output_wire(&wi, &wj, &gate, &self.index);
        let tt = self.get_tt(&wi, &wj, &wo, &gate);
        match gate {
            GateType::AND=> {
                let table = generate_table(&tt,  &self.index);
                let gate = Gate { gate_type: GateType::AND, table, wi, wj, wo };
                self.increment_index();
                gate
            }
            GateType::NAND=> {
                let table = generate_table(&tt,  &self.index);
                let gate = Gate { gate_type: GateType::NAND, table, wi, wj, wo };
                self.increment_index();
                gate
            }
            GateType::XOR=>Gate { gate_type: GateType::XOR, table: Vec::new(), wi, wj, wo },
            GateType::XNOR=>Gate { gate_type: GateType::XNOR, table: Vec::new(), wi, wj, wo },
            GateType::OR=> {
                let table = generate_table(&tt,  &self.index);
                let gate = Gate { gate_type: GateType::OR, table, wi, wj, wo };
                self.increment_index();
                gate
            },
            GateType::NOR=> {
                let table = generate_table(&tt,  &self.index);
                let gate = Gate { gate_type: GateType::NOR, table, wi, wj, wo };
                self.increment_index();
                gate
            },
        }
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
    fn reset_index(&mut self) {
        self.index = BigUint::from(0u32)
    }
}

fn generate_table(tt: &[(BigUint, BigUint, BigUint); 4], gate_id: &BigUint) -> Vec<BigUint> {
    let mut table = vec![BigUint::from(0u8); 3];
    // Creating symmetric key from left input, right input and gate id then encrypting the tt output with the key
    for (il, ir, out) in tt {
        let key = crypto_utils::gc_kdf_128(il, ir, gate_id);
        let ct = key ^ out;
        let pos = crate::gates::grr3_gate_gen::get_position(il, ir);
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