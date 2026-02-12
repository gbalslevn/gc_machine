use num_bigint::{BigUint, ToBigUint}; 
use rand::{Rng, thread_rng};
// https://docs.rs/num-bigint/latest/num_bigint/
use sha2::{Digest, Sha256};

// Derives a key from the two input labels and the gate_id in the gc
pub fn gc_kdf(left: &BigUint, right: &BigUint, gate_id: &BigUint) -> BigUint {
    let mut hasher = Sha256::new();
    hasher.update(left.to_bytes_be());
    hasher.update(right.to_bytes_be());
    hasher.update(gate_id.to_bytes_be());
    let bit_result = hasher.finalize(); // u32 bit result
    BigUint::from_bytes_be(&bit_result)
}

pub fn generate_label() -> BigUint {
    generate_128_bit_number()
}

fn generate_128_bit_number() -> BigUint {
    let mut bytes = [0u8; 16]; // 128 bits
    thread_rng().fill(&mut bytes);
    BigUint::from_bytes_be(&bytes)
}
