use crate::crypto_utils;
use num_bigint::{BigUint};

#[test]
fn label_is_128_bits() {
    let mut rng = crypto_utils::gen_rng();
    let label = crypto_utils::generate_label(&mut rng);
    assert!(label.to_bytes_be().len() == 16)
}
#[test]
fn label_is_non_deterministic() {
    let mut rng = crypto_utils::gen_rng();
    let l1 = crypto_utils::generate_label(&mut rng);
    let l2 = crypto_utils::generate_label(&mut rng);
    assert_ne!(l1, l2)
}
// Tests for gc_kdf
#[test]
fn kdf_is_deterministic() {
    let mut rng = crypto_utils::gen_rng();
    let l1 = crypto_utils::generate_label(&mut rng);
    let l2 = crypto_utils::generate_label(&mut rng);
    let gate_id = BigUint::ZERO;
    let key1 = crypto_utils::gc_kdf(&l1, &l2, &gate_id);
    let key2 = crypto_utils::gc_kdf(&l1, &l2, &gate_id);
    assert_eq!(key1, key2, "KDF must be deterministic!");
}
#[test]
fn kdf_output_is_256_bits() {
    let mut rng = crypto_utils::gen_rng();
    let l1 = crypto_utils::generate_label(&mut rng);
    let l2 = crypto_utils::generate_label(&mut rng);
    let gate_id = BigUint::ZERO;
    let key1 = crypto_utils::gc_kdf(&l1, &l2, &gate_id);
    assert_eq!(key1.to_bytes_be().len(), 32); // The key should be 256 bits (or 32 bytes), from SHA256
}

#[test]
fn kdf_is_unique() { // Should be unique, assuming no collision
    let mut rng = crypto_utils::gen_rng();
    let l1 = crypto_utils::generate_label(&mut rng);
    let l2 = crypto_utils::generate_label(&mut rng);
    let gate_id = BigUint::ZERO;
    let key1 = crypto_utils::gc_kdf(&l1, &l2, &gate_id);
    let key2 = crypto_utils::gc_kdf(&l2, &l1, &gate_id);
    assert_ne!(key1, key2, "KDF should vary with different label inputs.");
}

#[test]
// Tests gen bool can generate a true and a false
fn generates_bools_randomly() {
    let mut rng = crypto_utils::gen_rng();
    let val = crypto_utils::gen_bool(&mut rng);
    let mut neg_val = crypto_utils::gen_bool(&mut rng);
    while val == neg_val  {
        neg_val = crypto_utils::gen_bool(&mut rng)
    }
    assert!(val != neg_val)
}

#[test]
fn sha256_is_deterministic() {
    let h_0 = crypto_utils::sha256(&BigUint::ZERO.to_bytes_be());
    let h_1 = crypto_utils::sha256(&BigUint::ZERO.to_bytes_be());
    assert!(h_0 == h_1)
}