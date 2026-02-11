use crate::crypto_utils;
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use num_bigint::BigUint;

// Unit tests for crypto_utils

#[test]
fn it_adds_two() {
    assert_eq!(1+1, 2);
}

#[test]
fn kdf_is_deterministic() {
    let seed = [42u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);
    let l1 = crypto_utils::generate_label(&mut rng);
    let l2 = crypto_utils::generate_label(&mut rng);
    let gate_id = BigUint::ZERO;
    let key1 = crypto_utils::gc_kdf(&l1, &l2, &gate_id);
    let key2 = crypto_utils::gc_kdf(&l1, &l2, &gate_id);
    assert_eq!(key1, key2, "KDF must be deterministic!");
}