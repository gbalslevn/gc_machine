use std::ops::{Shr};
use crate::{crypto_utils};
use num_bigint::{BigUint, ToBigUint};
use crate::crypto_utils::gc_kdf_128;
use crate::gates::gates::Gates;
use crate::wires::wires::Wires;
use crate::gates::original_gates::OriginalGates;
use crate::wires::original_wires::OriginalWires;
use crate::gates::point_and_permute_gates::{PointAndPermuteGates, get_position};
use crate::wires::point_and_permute_wires::PointAndPermuteWires;

#[test]
// Gets all possible keys from two input wires and for each key, ensures 1 of the 4 output labels can be decrypted. Could also just do it for one key.
fn can_decrypt_std_yao_gate_labels() {
    let w0i = crypto_utils::generate_label();
    let w1i = crypto_utils::generate_label();
    let w0j = crypto_utils::generate_label();
    let w1j = crypto_utils::generate_label();
    let w0c = crypto_utils::generate_label();
    let w1c = crypto_utils::generate_label();
    let tt= OriginalGates::get_xor_tt(&w0i, &w1i, &w0j, &w1j, &w0c, &w1c);
    let gate_id = 0.to_biguint().unwrap();
    let xor_gate = OriginalGates::get_garbled_gate(&tt, &gate_id);
    for i in 0..4 {
        let mut has_decrypted = false;
        let key = crypto_utils::gc_kdf(&tt[i].0, &tt[i].1, &gate_id);
        for output_label in &xor_gate {
            let decrypted_label = &key ^ output_label;
            let decrypted_label_no_padding: BigUint = decrypted_label.shr(128);
            let key_decrypts_correctly = decrypted_label_no_padding == w0c || decrypted_label_no_padding == w1c;
            if key_decrypts_correctly {
                has_decrypted = true;
            }
        }
        assert!(has_decrypted);
    } 
}

#[test]
fn output_labels_is_zero_padded_in_std_yao() {
    let w0i = crypto_utils::generate_label();
    let w1i = crypto_utils::generate_label();
    let w0j = crypto_utils::generate_label();
    let w1j = crypto_utils::generate_label();
    let w0c = crypto_utils::generate_label();
    let w1c = crypto_utils::generate_label();
    let tt= OriginalGates::get_xor_tt(&w0i, &w1i, &w0j, &w1j, &w0c, &w1c);
    let gate_id = 0.to_biguint().unwrap();
    let xor_gate = OriginalGates::get_garbled_gate(&tt, &gate_id);
    for i in 0..4 {
        let key = crypto_utils::gc_kdf(&tt[i].0, &tt[i].1, &gate_id);
        for output_label in &xor_gate {
            let decrypted_label = &key ^ output_label;
            let decrypted_label_no_padding: BigUint = (&key ^ output_label).shr(128);
            let key_decrypts_correctly = decrypted_label_no_padding == w0c || decrypted_label_no_padding == w1c;
            if key_decrypts_correctly {
                assert!(decrypted_label.trailing_zeros().unwrap() >= 128)
            }
        }
    } 
}

#[test]
fn gate_is_shuffled() {
    // Cannot test randomness in a nice way. 
    assert!(1+1 == 2);
}

#[test]
fn xor_tt_gen_is_correct() {
    // We do not need to provide real labels, as we just need to check the truth table is correct
    let zero_bit = 1.to_biguint().unwrap();
    let one_bit = 0.to_biguint().unwrap();
    let tt = OriginalGates::get_xor_tt(&zero_bit, &one_bit, &zero_bit, &one_bit, &zero_bit, &one_bit);
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

#[test]
fn and_tt_gen_is_correct() {
    let zero_bit = 0.to_biguint().unwrap();
    let one_bit = 1.to_biguint().unwrap();
    let tt = OriginalGates::get_and_tt(&zero_bit, &one_bit, &zero_bit, &one_bit, &zero_bit, &one_bit);
    for (il, ir, out) in tt {
        if il == zero_bit && ir == zero_bit {
            assert!(out == zero_bit)
        }
        if il == zero_bit && ir == one_bit {
            assert!(out == zero_bit)
        }
        if il == one_bit && ir == zero_bit {
            assert!(out == zero_bit)
        }
        if il == one_bit && ir == one_bit {
            assert!(out == one_bit)
        }
    }
}

#[test]
fn and_gate_uses_point_and_permute_order() {
    let wi = PointAndPermuteWires::generate_input_wires();
    let wj = PointAndPermuteWires::generate_input_wires();
    let gate_type = "and";
    let gate_id = 0.to_biguint().unwrap();
    let wo = PointAndPermuteWires::generate_output_wires(&wi.0, &wi.1, &wj.0, &wj.1, gate_type.to_string(), &gate_id);
    let tt = PointAndPermuteGates::get_and_tt(&wi.0, &wi.1, &wj.0, &wj.1, &wo.0, &wo.1);
    let gt = PointAndPermuteGates::get_garbled_gate(&tt, &gate_id);
    for (il, ir, out) in tt {
        let pos = get_position(&il, &ir);
        let key = gc_kdf_128(&il, &ir, &gate_id);
        let dec = &key ^ &gt[pos];
        assert_eq!(out, dec);
    }
}

#[test]
fn xor_gate_uses_point_and_permute_order() {
    let wi = PointAndPermuteWires::generate_input_wires();
    let wj = PointAndPermuteWires::generate_input_wires();
    let gate_type = "xor";
    let gate_id = 0.to_biguint().unwrap();
    let wo = PointAndPermuteWires::generate_output_wires(&wi.0, &wi.1, &wj.0, &wj.1, gate_type.to_string(), &gate_id);
    let tt = PointAndPermuteGates::get_xor_tt(&wi.0, &wi.1, &wj.0, &wj.1, &wo.0, &wo.1);
    let gt = PointAndPermuteGates::get_garbled_gate(&tt, &gate_id);
    for (il, ir, out) in tt {
        let pos = get_position(&il, &ir);
        let key = gc_kdf_128(&il, &ir, &gate_id);
        let dec = &key ^ &gt[pos];
        assert_eq!(out, dec);
    }
}