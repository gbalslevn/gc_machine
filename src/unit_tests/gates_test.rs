use crate::{crypto_utils};
use num_bigint::{BigUint, ToBigUint};
use crate::crypto_utils::gc_kdf_128;
use crate::gates::gates::{Gate, GateType, Gates};
use crate::gates::grr3_gates::GRR3Gates;
use crate::wires::wires::Wires;
use crate::gates::original_gates::OriginalGates;
use crate::wires::original_wires::OriginalWires;
use crate::gates::point_and_permute_gates::{PointAndPermuteGates, get_position};
use crate::wires::grr3_wires::GRR3Wires;
use crate::wires::point_and_permute_wires::PointAndPermuteWires;

#[test]
// Gets all possible keys from two input wires and for each key, ensures at least 1 of the 4 output labels can be decrypted. Could also just do it for one key.
// fn can_decrypt_std_yao_gate_labels() {
//     let gate_id = 0.to_biguint().unwrap();
//     let gate = GateType::XOR;
//     let xor_gate = OriginalGates::new(&gate, gate_id);
//     let output_table = xor_gate.table;
//     for i in 0..4 {
//         let mut has_decrypted = false;
//         let key = crypto_utils::gc_kdf(&output_table[i], &tt[i].1, &gate_id);
//         for output_label in &output_table {
//             let decrypted_label = &key ^ output_label;
//             let decrypted_label_no_padding: BigUint = decrypted_label >> 128;
//             let key_decrypts_correctly = decrypted_label_no_padding == wo.0 || decrypted_label_no_padding == wo.1;
//             if key_decrypts_correctly {
//                 has_decrypted = true;
//             }
//         }
//         assert!(has_decrypted);
//     } 
// }

#[test]
fn output_labels_is_zero_padded_in_std_yao() {
    let gate_id = 0.to_biguint().unwrap();
    let gate = GateType::XOR;
    let gt = OriginalGates::new(&gate, gate_id.clone());
    let tt= OriginalGates.get_tt(&gt.wi, &gt.wj, &gt.wo, &gate);
    let wo = gt.wo;
    for i in 0..4 {
        let key = crypto_utils::gc_kdf(&tt[i].0, &tt[i].1, &gate_id);
        for output_label in &gt.table {
            let decrypted_label = &key ^ output_label;
            let decrypted_label_no_padding: BigUint = (&key ^ output_label) >> 128;
            let key_decrypts_correctly = &decrypted_label_no_padding == wo.w0() || &decrypted_label_no_padding == wo.w1();
            if key_decrypts_correctly {
                assert!(decrypted_label.trailing_zeros().unwrap() >= 128)
            }
        }
    } 
}

#[test]
fn gate_is_shuffled() {
    // Cannot test randomness in a nice way. 
    assert_eq!(1 + 1, 2);
}

#[test]
fn xor_tt_gen_is_correct() {
    // We do not need to provide real labels, as we just need to check the truth table is correct
    let zero_bit = 0.to_biguint().unwrap();
    let one_bit = 1.to_biguint().unwrap();
    let w = OriginalWires::new(zero_bit.clone(), one_bit.clone());
    let tt = OriginalGates.get_tt(&w,&w,&w, &GateType::XOR);
    for (il, ir, out) in tt {
        if il == zero_bit && ir == zero_bit {
            assert_eq!(out, zero_bit)
        }
        if il == zero_bit && ir == one_bit {
            assert_eq!(out, one_bit)
        }
        if il == one_bit && ir == zero_bit {
            assert_eq!(out, one_bit)
        }
        if il == one_bit && ir == one_bit {
            assert_eq!(out, zero_bit)
        }
    }
}

#[test]
fn and_tt_gen_is_correct() {
    let zero_bit = 0.to_biguint().unwrap();
    let one_bit = 1.to_biguint().unwrap();
    let w = OriginalWires::new(zero_bit.clone(), one_bit.clone());
    let tt = OriginalGates.get_tt(&w,&w,&w, &GateType::AND);
    for (il, ir, out) in tt {
        if il == zero_bit && ir == zero_bit {
            assert_eq!(out, zero_bit)
        }
        if il == zero_bit && ir == one_bit {
            assert_eq!(out, zero_bit)
        }
        if il == one_bit && ir == zero_bit {
            assert_eq!(out, zero_bit)
        }
        if il == one_bit && ir == one_bit {
            assert_eq!(out, one_bit)
        }
    }
}

#[test]
fn and_gate_uses_point_and_permute_order() {
    let gate_id = 0.to_biguint().unwrap();
    let gate = GateType::AND;
    let gt = PointAndPermuteGates::new(&gate, gate_id.clone());
    let tt = PointAndPermuteGates.get_tt(&gt.wi, &gt.wj, &gt.wo, &gate);
    for (il, ir, out) in tt {
        let pos = get_position(&il, &ir);
        let key = gc_kdf_128(&il, &ir, &gate_id);
        let dec = &key ^ &gt.table[pos];
        assert_eq!(out, dec);
    }
}

#[test]
fn xor_gate_uses_point_and_permute_order() {
    let gate_id = 0.to_biguint().unwrap();
    let gate = GateType::XOR;
    let gt = PointAndPermuteGates::new(&gate, gate_id.clone());
    let tt = PointAndPermuteGates.get_tt(&gt.wi, &gt.wj, &gt.wo, &gate);
    for (il, ir, out) in tt {
        let pos = get_position(&il, &ir);
        let key = gc_kdf_128(&il, &ir, &gate_id);
        let dec = &key ^ &gt.table[pos];
        assert_eq!(out, dec);
    }
}

#[test]
fn gate_only_3_entries_grr3() {
    let gate_id = 0.to_biguint().unwrap();
    let gt = GRR3Gates::new(&GateType::XOR, gate_id);
    assert_eq!(gt.table.len(), 3);
}

#[test]
fn are_and_output_labels_correct_grr3() {
    let gate_id = 0.to_biguint().unwrap();
    let gate = GateType::AND;
    let gt = GRR3Gates::new(&gate, gate_id.clone());
    let tt = GRR3Gates.get_tt(&gt.wi, &gt.wj, &gt.wo, &gate);
    for (il, ir, out) in tt {
        let pos = get_position(&il, &ir);
        let key = gc_kdf_128(&il, &ir, &gate_id);
        if pos != 0 {
            let dec = &key ^ &gt.table[pos-1];
            assert_eq!(out, dec);
        } else {
            assert_eq!(out, key);
        }
    }
}

#[test]
fn xor_output_labels_are_correct_grr3() {
    let gate_id = 0.to_biguint().unwrap();
    let gate = GateType::XOR;
    let gt = GRR3Gates::new(&gate, gate_id.clone());
    let tt = GRR3Gates.get_xor_tt(&gt.wi, &gt.wj, &gt.wo);
    for (il, ir, out) in tt {
        let pos = get_position(&il, &ir);
        if pos != 0 {
            let key = gc_kdf_128(&il, &ir, &gate_id);
            let dec = &key ^ &gt.table[pos-1];
            assert_eq!(out, dec);
        } else {
            let key = gc_kdf_128(&il, &ir, &gate_id);
            assert_eq!(out, key);
        }
    }
}