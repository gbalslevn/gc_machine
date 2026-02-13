use num_bigint::{BigUint, ToBigUint}; 
use rand::{Rng, thread_rng};
// https://docs.rs/num-bigint/latest/num_bigint/
use sha2::{Digest, Sha256};

// Derives a key from the two input labels and the gate_id in the gc
pub fn gc_kdf(left: &BigUint, right: &BigUint, gate_id: &BigUint) -> BigUint {
    let mut hasher = Sha256::new();
    hasher.update(left.to_bytes_le());
    hasher.update(right.to_bytes_le());
    hasher.update(gate_id.to_bytes_le());
    let bit_result = hasher.finalize(); // u32 bit result
    BigUint::from_bytes_be(&bit_result)
}

pub fn gc_kdf_128(left: &BigUint, right: &BigUint, gate_id: &BigUint) -> BigUint {
    let bit_result = gc_kdf(left, right, gate_id);
    bit_result >> 128
}

pub fn generate_label_lsb(lsb: bool) -> BigUint {
    let mut bytes = [0u8; 16];
    thread_rng().fill(&mut bytes);
    if lsb {
        bytes[15] |= 1;  // LSB = 1
    } else {
        bytes[15] &= !1; // LSB = 0
    }
    BigUint::from_bytes_be(&bytes)
}

pub fn generate_label() -> BigUint {
    generate_128_bit_number()
}

pub fn get_biguint(number : u16) -> BigUint {
    number.to_biguint().unwrap()
}

fn generate_128_bit_number() -> BigUint {
    let mut bytes = [0u8; 16]; // 128 bits
    thread_rng().fill(&mut bytes);
    BigUint::from_bytes_be(&bytes)
}
