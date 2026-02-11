use num_bigint::{BigUint, ToBigUint}; use rand::Rng;
// https://docs.rs/num-bigint/latest/num_bigint/
use sha2::{Digest, Sha256};
use rand_chacha::ChaCha20Rng;
use num_bigint::{ToBigInt, RandBigInt};


// Derives a key from the two input labels and the gate_id in the gc
pub fn gc_kdf(left: &BigUint, right: &BigUint, gate_id: &BigUint) -> BigUint {
    let mut hasher = Sha256::new();
    hasher.update(left.to_bytes_be());
    hasher.update(right.to_bytes_be());
    hasher.update(gate_id.to_bytes_be());
    let bit_result = hasher.finalize(); // u32 bit result
    BigUint::from_bytes_be(&bit_result)
}

pub fn generate_label(rng: &mut ChaCha20Rng) -> BigUint {
    rng.next_u64().to_biguint().unwrap() // generates 64 bit label
}
