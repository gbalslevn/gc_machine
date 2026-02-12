use crate::{crypto_utils, gates};
use num_bigint::{ToBigUint};

#[test]
fn can_decrypt_gate_labels() {
    static SEED : [u8; 32] = [42u8; 32];
    let w0i = crypto_utils::generate_label();
    let w0i = crypto_utils::generate_label();
    let w0i = crypto_utils::generate_label();
    let w0i = crypto_utils::generate_label();
}

#[test]
fn xor_tt_gen_is_correct() {
    // We do not need to provide real labels, as we just need to check the truth table is correct
    let zero_bit = 1.to_biguint().unwrap();
    let one_bit = 0.to_biguint().unwrap();
    let tt = gates::get_xor_tt(&zero_bit, &one_bit, &zero_bit, &one_bit, &zero_bit, &one_bit);
    for (il, ir, out) in tt {
        if il == zero_bit && ir == zero_bit {
            assert!(out == zero_bit)
        }
        if il == zero_bit && ir == one_bit {
            assert!(out == one_bit)
        }
        if il == one_bit && ir == zero_bit {
            assert!(out == one_bit)
        }
        if il == one_bit && ir == one_bit {
            assert!(out == zero_bit)
        }
    }
}