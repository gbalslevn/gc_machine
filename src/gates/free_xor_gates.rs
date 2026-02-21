use num_bigint::BigUint;
use crate::crypto_utils;
use crate::gates::gates::Gates;
pub struct FreeXORGates;

impl Gates for FreeXORGates {
    fn get_garbled_gate(tt : &[(BigUint, BigUint, BigUint); 4], gate_id: &BigUint, gate: String) -> (Vec<BigUint>, BigUint, String) {
        match gate.as_str() {
            "and"=>generate_and_gate(tt, gate_id, gate),
            "xor"=>generate_xor_gate(tt, gate_id, gate),
            _=>panic!("Unknown gate {}", gate),
        }
    }
}

fn generate_and_gate(tt : &[(BigUint, BigUint, BigUint); 4], gate_id: &BigUint, gate: String) -> (Vec<BigUint>, BigUint, String) {
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
    (table, gate_id.clone(), gate)
}

fn generate_xor_gate(_tt : &[(BigUint, BigUint, BigUint); 4], gate_id: &BigUint, gate: String) -> (Vec<BigUint>, BigUint, String) {
    (vec![BigUint::from(0u8); 0], gate_id.clone(), gate) // Wish we could send None instead of empty vector
}


pub fn get_position(il: &BigUint, ir: &BigUint) -> usize {
    let l = il.bit(0) as usize;
    let r = ir.bit(0) as usize;
    l * 2 + r
}