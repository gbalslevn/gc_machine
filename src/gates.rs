use num_bigint::BigUint;
use crate::crypto_utils;
use rand::{thread_rng};
use rand::seq::SliceRandom;

// Get shuffled gabled gate by providing the truth table for it. 
pub fn get_garbled_gate(tt : &[(BigUint, BigUint, BigUint); 4], gate_id: &BigUint) -> Vec<BigUint> {
    let mut table = Vec::new();
    // Creating symmetric key from left input, right input and gate id then encrypting the tt output with the key 
    for (il, ir, out) in tt {
        let key = crypto_utils::gc_kdf(il, ir, gate_id);
        let ct = key ^ out; 
        table.push(ct);
    }
    table.shuffle(&mut thread_rng());
    table
}

/// Generates a 2-input XOR truth table from provided wire labels
/// // # Arguments
/// * `w0i` - Input wire *i* representing bit 0.
/// * `w1i` - Input wire *i* representing bit 1.
/// * `w0j` - Input wire *j* representing bit 0.
/// * `w1j` - Input wire *j* representing bit 1.
/// * `w0c` - Output wire *o* representing bit 0.
/// * `w1c` - Output wire *o* representing bit 1.
pub fn get_xor_tt(w0i: &BigUint, w1i: &BigUint, w0j: &BigUint, w1j: &BigUint, w0c: &BigUint, w1c: &BigUint) -> [(BigUint, BigUint, BigUint); 4] {
    [(w0i.clone(), w0j.clone(), w0c.clone()), (w0i.clone(), w1j.clone(), w1c.clone()), (w1i.clone(), w0j.clone(), w1c.clone()), (w1i.clone(), w1j.clone(), w0c.clone())] // should avoid using clone if wanting performancee
}

pub fn get_and_tt(w0i: &BigUint, w1i: &BigUint, w0j: &BigUint, w1j: &BigUint, w0c: &BigUint, w1c: &BigUint) -> [(BigUint, BigUint, BigUint); 4] {
    [(w0i.clone(), w0j.clone(), w0c.clone()), (w0i.clone(), w1j.clone(), w0c.clone()), (w1i.clone(), w0j.clone(), w0c.clone()), (w1i.clone(), w1j.clone(), w1c.clone())] 
}

// Build NOT gate using XOR with constant 1

// Build OR gate: A ∨ B = (A ⊕ B) ⊕ (A ∧ B)