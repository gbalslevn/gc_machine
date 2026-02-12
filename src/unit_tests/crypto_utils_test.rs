use crate::crypto_utils;
use num_bigint::BigUint;

// Tests for gc_kdf
#[test]
fn kdf_is_deterministic() {
    let l1 = crypto_utils::generate_label();
    let l2 = crypto_utils::generate_label();
    let gate_id = BigUint::ZERO;
    let key1 = crypto_utils::gc_kdf(&l1, &l2, &gate_id);
    let key2 = crypto_utils::gc_kdf(&l1, &l2, &gate_id);
    assert!(key1.bits() == 256); // The key should be 256 bits, from SHA256
    assert_eq!(key1, key2, "KDF must be deterministic!");
}
#[test]
fn kdf_is_unique() { // Should be unique, assuming no collision
    let l1 = crypto_utils::generate_label();
    let l2 = crypto_utils::generate_label();
    let gate_id = BigUint::ZERO;
    let key1 = crypto_utils::gc_kdf(&l1, &l2, &gate_id);
    let key2 = crypto_utils::gc_kdf(&l2, &l1, &gate_id);
    assert!(key1.bits() == 256); // The key should be 256 bits, from SHA256
    assert_ne!(key1, key2, "KDF should vary with different label inputs.");
}