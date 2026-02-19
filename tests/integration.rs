use std::ops::Shr;
use gc_machine::crypto_utils;
use gc_machine::gates::gates::Gates;
use gc_machine::gates::original_gates::OriginalGates;
use num_bigint::BigUint;
use gc_machine::wires::original_wires::OriginalWires;
use gc_machine::wires::wires::Wires;

#[test]
// Sanity test
fn one_plus_one_is_two() {
    assert_eq!(1 + 1, 2);
} 

#[test]
// Garbler and Evaluator each provides a bit and can compare them using the standard yao garbled circuit. 
fn can_compare_a_bit_using_std_yao() {
    let wires = OriginalWires;
    // 1. Garbler creates circuit, a single XOR gate, and sends to evaluator
    let wi = wires.generate_input_wires();
    let wj = wires.generate_input_wires();
    let gate_id = BigUint::ZERO;
    let wo = wires.generate_output_wires(&wi, &wj, "xor".to_string(), &gate_id);
    let tt = OriginalGates::get_xor_tt(&wi, &wj, &wo);
    let xor_gate = OriginalGates::get_garbled_gate(&tt, &gate_id);
    
    // 2. Evaluator receives circuit and chooses which bit-label he wants using oblivious transfer. 
    // * OT MAGIC *
    // 3. Garbler sends her bit as a label as well as the evaluators labels. Evaluator now has what is needed to evaluate.
    let g_label = wi.0;
    let e_label = wj.1;
    let mut decrypted_output_label =  BigUint::ZERO;
    let key = crypto_utils::gc_kdf(&g_label, &e_label, &gate_id);
    for ct in xor_gate {
        let label = ct ^ &key;
        if label.trailing_zeros().unwrap() >= 128 {
            decrypted_output_label = label;
            break;
        } 
    }
    let decrypted_output_label_no_padding = decrypted_output_label.shr(128);
    // Garbler has sent a lookup table which evaluator now uses to see the result. Here we know the result should be 1, as 0 ^ 1 = 1
    assert_eq!(decrypted_output_label_no_padding, wo.1);
} 