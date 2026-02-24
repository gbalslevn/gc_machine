use num_bigint::BigUint;
use crate::crypto_utils;
use crate::gates::gates::{Gate, GateType, Gates};
use crate::wires::free_xor_wires::FreeXORWires;
use crate::wires::wires::{Wire, Wires};
pub struct FreeXORGates;

// Implements free XOR and grr3

impl Gates for FreeXORGates {
    fn new(gate : GateType, wi: Wire, wj: Wire, gate_id: BigUint) -> Gate {
        let wo = FreeXORWires::generate_output_wire(&wi, &wj, &gate, &gate_id);
        let tt = FreeXORGates.get_tt(&wi, &wj, &wo, &gate);
        match gate {
            GateType::AND=> {
                let table = generate_and_table(&tt,  &gate_id);
                Gate { gate_id, gate_type: GateType::AND, table, wi: wi, wj: wj, wo: wo }
            }
            GateType::XOR=>Gate { gate_id, gate_type: GateType::XOR, table: Vec::new(), wi: wi, wj: wj, wo: wo }
        }
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