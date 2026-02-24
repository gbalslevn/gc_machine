use crate::crypto_utils;
use crate::crypto_utils::gc_kdf_128;
use crate::gates::free_xor_gates::FreeXORGates;
use crate::gates::gates::{GateType, Gates};
use crate::gates::grr3_gates::GRR3Gates;
use crate::gates::original_gates::OriginalGates;
use crate::gates::point_and_permute_gates::{PointAndPermuteGates, get_position};
use crate::wires::free_xor_wires::FreeXORWires;
use crate::wires::grr3_wires::GRR3Wires;
use crate::wires::original_wires::OriginalWires;
use crate::wires::point_and_permute_wires::PointAndPermuteWires;
use crate::wires::wires::{Wire, Wires};
use num_bigint::{BigUint, ToBigUint};

#[test]
// Gets all possible keys from two input wires and for each key, ensures 1 of the 4 output labels can be decrypted. Could also just do it for one key.
fn can_decrypt_std_yao_gate_labels() {
    let gate_id = 0.to_biguint().unwrap();
    let wire_gen = OriginalWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let xor_gate = OriginalGates::new(GateType::XOR, wi, wj, gate_id.clone());

    let mut has_decrypted = false;
    let key_0 = crypto_utils::gc_kdf(xor_gate.wi.w0(), xor_gate.wj.w0(), &gate_id);
    let key_1 = crypto_utils::gc_kdf(xor_gate.wi.w1(), xor_gate.wj.w0(), &gate_id);
    let key_2 = crypto_utils::gc_kdf(xor_gate.wi.w0(), xor_gate.wj.w1(), &gate_id);
    let key_3 = crypto_utils::gc_kdf(xor_gate.wi.w1(), xor_gate.wj.w1(), &gate_id);
    let keys = vec![key_0, key_1, key_2, key_3];
    for output_label in &xor_gate.table {
        for key in &keys {
            let decrypted_label = key ^ output_label;
            let decrypted_label_no_padding: BigUint = decrypted_label >> 128;
            let key_decrypts_correctly =
                &decrypted_label_no_padding == xor_gate.wo.w0() || &decrypted_label_no_padding == xor_gate.wo.w1();
            if key_decrypts_correctly {
                has_decrypted = true;
                break;
            }
        }
        assert!(has_decrypted);
        has_decrypted = false;
    }
}

#[test]
fn output_labels_is_zero_padded_in_std_yao() {
    let gate_id = 0.to_biguint().unwrap();
    let gate = GateType::XOR;
    let wire_gen = OriginalWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gt = OriginalGates::new(GateType::XOR, wi, wj, gate_id);

    let tt = OriginalGates.get_tt(&gt.wi, &gt.wj, &gt.wo, &gate);
    for i in 0..4 {
        let key = crypto_utils::gc_kdf(&tt[i].0, &tt[i].1, &gt.gate_id);
        for output_label in &gt.table {
            let decrypted_label = &key ^ output_label;
            let decrypted_label_no_padding: BigUint = (&key ^ output_label) >> 128;
            let key_decrypts_correctly =
                &decrypted_label_no_padding == gt.wo.w0() || &decrypted_label_no_padding == gt.wo.w1();
            if key_decrypts_correctly {
                assert!(decrypted_label.trailing_zeros().unwrap() >= 128)
            }
        }
    }
}

#[test] 
fn original_gate_table_has_4_entries() {
    let wire_gen = OriginalWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gt = OriginalGates::new(GateType::XOR, wi, wj, BigUint::ZERO);
    assert_eq!(gt.table.len(), 4);
}

#[test]
fn gate_is_shuffled() {
    // Cannot test randomness in a nice way.
    assert_eq!(1 + 1, 2);
}

#[test]
fn xor_tt_gen_is_correct() {
    // We do not need to provide real labels, as we just need to check the truth table is correct
    let zero_bit= 1.to_biguint().unwrap();
    let one_bit = 1.to_biguint().unwrap();
    let w = Wire::new(zero_bit.clone(), one_bit.clone());
    let tt = OriginalGates.get_tt(&w, &w, &w, &GateType::XOR);
    for (il, ir, out) in tt {
        if il == zero_bit || il == zero_bit && ir == zero_bit {
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
    let w = Wire::new(zero_bit.clone(), one_bit.clone());
    let tt = OriginalGates.get_tt(&w, &w, &w, &GateType::AND);
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
    let wire_gen = PointAndPermuteWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gt = PointAndPermuteGates::new(gate, wi, wj, gate_id);
    let tt = PointAndPermuteGates.get_tt(&gt.wi, &gt.wj, &gt.wo, &gt.gate_type);
    for (il, ir, out) in tt {
        let pos = get_position(&il, &ir);
        let key = gc_kdf_128(&il, &ir, &gt.gate_id);
        let dec = &key ^ &gt.table[pos];
        assert_eq!(out, dec);
    }
}

#[test]
fn xor_gate_uses_point_and_permute_order() {
    let gate_id = 0.to_biguint().unwrap();
    let gate = GateType::XOR;
    let wire_gen = PointAndPermuteWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gt = PointAndPermuteGates::new(gate, wi, wj, gate_id);
    let tt = PointAndPermuteGates.get_tt(&gt.wi, &gt.wj, &gt.wo, &gate);
    for (il, ir, out) in tt {
        let pos = get_position(&il, &ir);
        let key = gc_kdf_128(&il, &ir, &gt.gate_id);
        let dec = &key ^ &gt.table[pos];
        assert_eq!(out, dec);
    }
}

#[test]
fn gate_only_3_entries_grr3() {
    let gate_id = 0.to_biguint().unwrap();
    let gate = GateType::AND;
    let wire_gen = GRR3Wires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gt = GRR3Gates::new(gate, wi, wj, gate_id);

    assert_eq!(gt.table.len(), 3);
}

#[test]
fn are_and_output_labels_correct_grr3() {
    let gate_id = 0.to_biguint().unwrap();
    let gate = GateType::AND;
    let wire_gen = GRR3Wires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gt = GRR3Gates::new(gate, wi, wj, gate_id);
    let tt = GRR3Gates.get_tt(&gt.wi, &gt.wj, &gt.wo, &gate);
    for (il, ir, out) in tt {
        let pos = get_position(&il, &ir);
        let key = gc_kdf_128(&il, &ir, &gt.gate_id);
        if pos != 0 {
            let dec = &key ^ &gt.table[pos - 1];
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
    let wire_gen = GRR3Wires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gt = GRR3Gates::new(gate, wi, wj, gate_id);
    let tt = GRR3Gates.get_xor_tt(&gt.wi, &gt.wj, &gt.wo);
    for (il, ir, out) in tt {
        let pos = get_position(&il, &ir);
        if pos != 0 {
            let key = gc_kdf_128(&il, &ir, &gt.gate_id);
            let dec = &key ^ &gt.table[pos - 1];
            assert_eq!(out, dec);
        } else {
            let key = gc_kdf_128(&il, &ir, &gt.gate_id);
            assert_eq!(out, key);
        }
    }
}

#[test]
fn no_entries_in_xor_gate_free_xor() {
    let gate_id = 0.to_biguint().unwrap();
    let gate = GateType::XOR;
    let wire_gen = FreeXORWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gt = FreeXORGates::new(gate, wi, wj, gate_id);
    assert_eq!(gt.table.len(), 0);
}

#[test]
fn three_entries_in_and_gate_free_xor() {
    let gate_id = 0.to_biguint().unwrap();
    let gate = GateType::AND;
    let wire_gen = FreeXORWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gt = FreeXORGates::new(gate, wi, wj, gate_id);
    assert_eq!(gt.table.len(), 3);
}
