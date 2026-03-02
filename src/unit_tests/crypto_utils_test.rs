use crate::crypto_utils;
use num_bigint::{BigUint};

#[test]
fn label_is_128_bits() {
    let label = crypto_utils::generate_label();
    assert!(label.to_bytes_be().len() == 16)
}
#[test]
fn label_is_non_deterministic() {
    let label1 = crypto_utils::generate_label();
    let label2 = crypto_utils::generate_label();
    assert_ne!(label1, label2)
}
// Tests for gc_kdf
#[test]
fn kdf_is_deterministic() {
    let l1 = crypto_utils::generate_label();
    let l2 = crypto_utils::generate_label();
    let gate_id = BigUint::ZERO;
    let key1 = crypto_utils::gc_kdf(&l1, &l2, &gate_id);
    let key2 = crypto_utils::gc_kdf(&l1, &l2, &gate_id);
    assert_eq!(key1, key2, "KDF must be deterministic!");
}
#[test]
fn kdf_output_is_256_bits() {
    let l1 = crypto_utils::generate_label();
    let l2 = crypto_utils::generate_label();
    let gate_id = BigUint::ZERO;
    let key1 = crypto_utils::gc_kdf(&l1, &l2, &gate_id);
    assert_eq!(key1.to_bytes_be().len(), 32); // The key should be 256 bits (or 32 bytes), from SHA256
}

#[test]
fn kdf_is_unique() { // Should be unique, assuming no collision
    let l1 = crypto_utils::generate_label();
    let l2 = crypto_utils::generate_label();
    let gate_id = BigUint::ZERO;
    let key1 = crypto_utils::gc_kdf(&l1, &l2, &gate_id);
    let key2 = crypto_utils::gc_kdf(&l2, &l1, &gate_id);
    assert_ne!(key1, key2, "KDF should vary with different label inputs.");
}