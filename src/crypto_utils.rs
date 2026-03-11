use num_bigint::{BigUint, ToBigUint};
use sha2::{Digest, Sha256};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

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

pub fn gc_kdf_hg(wire: &BigUint, gate_id: &BigUint) -> BigUint {
    let mut hasher = Sha256::new();
    hasher.update(wire.to_bytes_le());
    hasher.update(gate_id.to_bytes_le());
    let bit_result = hasher.finalize();
    BigUint::from_bytes_be(&bit_result) >> 128
}

pub fn generate_label_lsb(rng: &mut ChaCha20Rng, lsb: bool) -> BigUint {
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes);
    if lsb {
        bytes[15] |= 1; // LSB = 1
    } else {
        bytes[15] &= !1; // LSB = 0
    }
    BigUint::from_bytes_be(&bytes)
}

pub fn gen_bool(rng : &mut ChaCha20Rng) -> bool {   
    let mut byte = [0u8; 1];
    rng.fill_bytes(&mut byte);
    byte[0] % 2 != 0
}

pub fn generate_label(rng : &mut ChaCha20Rng) -> BigUint {
    generate_128_bit_number(rng)
}

pub fn get_biguint(number: u16) -> BigUint {
    number.to_biguint().unwrap()
}

pub fn gen_rng() -> ChaCha20Rng {
    let seed = gen_seed();
    ChaCha20Rng::from_seed(seed)
}

pub fn sha256(data : &[u8]) -> BigUint {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    BigUint::from_bytes_be(&hash)
}

pub fn get_magic_number() -> BigUint {
    let ksdh = "AGF";
    let ksdh_biguint = BigUint::from_bytes_le(ksdh.as_bytes());
    ksdh_biguint
}

fn gen_seed() -> [u8; 32] {
    let mut buf = [0u8; 32];
    let _result = getrandom::fill(&mut buf);
    buf
}

fn generate_128_bit_number(rng : &mut ChaCha20Rng) -> BigUint {
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes);
    BigUint::from_bytes_be(&bytes)
}
