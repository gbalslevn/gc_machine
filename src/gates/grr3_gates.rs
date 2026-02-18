use num_bigint::BigUint;
use crate::crypto_utils;
use crate::gates::gates::Gates;

pub struct GRR3Gates;

impl Gates for GRR3Gates {
    fn get_garbled_gate(tt : &[(BigUint, BigUint, BigUint); 4], gate_id: &BigUint) -> Vec<BigUint> {
        let mut table = vec![BigUint::from(0u8); 3];
        // Creating symmetric key from left input, right input and gate id then encrypting the tt output with the key
        for (il, ir, out) in tt {
            let key = crypto_utils::gc_kdf_128(il, ir, gate_id);
            let ct = key ^ out;
            let pos = get_position(il, ir);
            if pos != 0 {
                table[pos-1] = ct;
            }
        }
        table
    }
}

pub fn get_position(il: &BigUint, ir: &BigUint) -> usize {
    let l = il.bit(0) as usize;
    let r = ir.bit(0) as usize;
    l * 2 + r
}