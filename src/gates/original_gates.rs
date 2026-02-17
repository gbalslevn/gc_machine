use std::ops::{BitXor, Shl};
use num_bigint::BigUint;
use crate::crypto_utils;
use rand::{thread_rng};
use rand::seq::SliceRandom;
use crate::gates::gates::Gates;

pub struct OriginalGates;

impl Gates for OriginalGates {
    fn get_garbled_gate(tt : &[(BigUint, BigUint, BigUint); 4], gate_id: &BigUint) -> Vec<BigUint> {
        let mut table = Vec::new();
        // Creating symmetric key from left input, right input and gate id then encrypting the tt output with the key
        for (il, ir, out) in tt {
            let key = crypto_utils::gc_kdf(il, ir, gate_id);
            let zero_padded_out = out.shl(128);
            let ct = key.bitxor(zero_padded_out);
            table.push(ct);
        }
        table.shuffle(&mut thread_rng());
        table
    }
}

