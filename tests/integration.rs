use std::ops::Shr;
use gc_machine::crypto_utils;
use gc_machine::gates::gates::{GateType, Gates};
use gc_machine::gates::original_gates::OriginalGates;
use gc_machine::wires::original_wires::OriginalWires;
use num_bigint::BigUint;
use gc_machine::ot::ot;
use gc_machine::ot::ot::encrypt;
use gc_machine::wires::wires::Wires;

#[test]
// Garbler (with wire wi) and Evaluator(with wire wj) each provides a bit and can compare them using the standard yao garbled circuit.
fn can_compare_a_bit_using_std_yao() {
    
    // 1. Garbler creates circuit, a single XOR gate, and sends to evaluator
    let gate_id = BigUint::ZERO;
    let wire_gen = OriginalWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let xor_gate = OriginalGates::new(GateType::XOR, wi, wj, gate_id);
    
    // 2. Evaluator receives circuit and chooses which bit-label he wants using OT.
    // 2.1 Evaluator prepares a ObliviousKeyPair and a RealKeyPar in that specific order, since he intends to receive the wirelabel for the 1-bit.
    let pp = ot::PublicParameters::new();
    let oblivious_keypair = ot::ObliviousKeyPair::new(&pp);
    let real_keypair = ot::RealKeyPair::new(&pp);
    // 2.2 The evaluator sends the publickey of both keypairs to the garbler who then encrypts wj.0 and wj.1 respectively.
    let _ciphertext_wj_0 = encrypt(&pp, oblivious_keypair.get_public_key(), xor_gate.wj.w0());
    let ciphertext_wj_1 = encrypt(&pp, real_keypair.get_public_key(), xor_gate.wj.w1());
    // 2.3 Upon receiving both ciphertexts, the evaluator can only succesfully decrypt the latter, which he does and sets g_label accordingly
    let e_label_received_from_ot = ot::decrypt(&pp, real_keypair.get_secret_key(),ciphertext_wj_1);
    // 3. Garbler sends her bit as a label (g_label) as well as the evaluators labels. Evaluator now has what is needed to evaluate.
    let g_label = xor_gate.wi.w0();
    let e_label = e_label_received_from_ot;
    let mut decrypted_output_label =  BigUint::ZERO;
    let key = crypto_utils::gc_kdf(&g_label, &e_label, &xor_gate.gate_id);
    for ct in xor_gate.table {
        let label = ct ^ &key;
        if label.trailing_zeros().unwrap() >= 128 {
            decrypted_output_label = label;
            break;
        } 
    }
    let decrypted_output_label_no_padding = decrypted_output_label.shr(128);
    // Garbler has sent a lookup table which evaluator now uses to see the result. Here we know the result should be 1, as 0 ^ 1 = 1
    assert_eq!(&decrypted_output_label_no_padding, xor_gate.wo.w1());
} 