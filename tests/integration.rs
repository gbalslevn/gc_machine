use std::ops::Shr;
use gc_machine::circuit_builder::WireBuild;
use gc_machine::evaluator::evaluator::Evaluator;
use gc_machine::evaluator::grr3_evaluator::GRR3Evaluator;
use gc_machine::evaluator::original_evaluator::OriginalEvaluator;
use gc_machine::evaluator::point_and_permute_evaluator::PointAndPermuteEvaluator;
use gc_machine::garbler::Garbler;
use gc_machine::gates::free_xor_gates::FreeXORGates;
use gc_machine::gates::grr3_gates::GRR3Gates;
use gc_machine::gates::point_and_permute_gates::PointAndPermuteGates;
use gc_machine::wires::free_xor_wires::FreeXORWires;
use gc_machine::wires::grr3_wires::GRR3Wires;
use gc_machine::wires::point_and_permute_wires::PointAndPermuteWires;
use gc_machine::{circuit_builder, crypto_utils, evaluator};
use gc_machine::gates::gates::{GateType, Gates};
use gc_machine::gates::original_gates::OriginalGates;
use gc_machine::wires::original_wires::OriginalWires;
use num_bigint::{BigUint, ToBigUint};
use gc_machine::ot::ot;
use gc_machine::ot::ot::encrypt;
use gc_machine::wires::wires::Wires;
use uuid::Uuid;

#[test]
// Garbler (with wire wi) and Evaluator(with wire wj) each provides a bit and can compare them using the standard yao garbled circuit.
fn can_compare_a_bit_using_std_yao() {
    
    // 1. Garbler creates circuit, a single XOR gate, and sends to evaluator
    let gate_id = BigUint::ZERO;
    let gate = GateType::XOR;
    let wire_gen = OriginalWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = OriginalGates::new(wire_gen);
    let xor_gate = gate_gen.generate_gate(gate, wi, wj, gate_id);
    // 2. Evaluator receives circuit and chooses which bit-label he wants using OT.
    // 2.1 Evaluator prepares a ObliviousKeyPair and a RealKeyPar in that specific order, since he intends to receive the wirelabel for the 1-bit.
    let oblivious_pp = ot::PublicParameters::new();
    let oblivious_keypair = ot::ObliviousKeyPair::new(&oblivious_pp);
    let real_pp = ot::PublicParameters::new();
    let real_keypair = ot::RealKeyPair::new(&real_pp);
    // 2.2 The evaluator sends the publickey of both keypairs to the garbler who then encrypts wj.0 and wj.1 respectively.
    let _ciphertext_wj_0 = encrypt(&oblivious_pp, &oblivious_keypair.get_public_key(), xor_gate.wj.w0());
    let ciphertext_wj_1 = encrypt(&real_pp, &real_keypair.get_public_key(), xor_gate.wj.w1());
    // 2.3 Upon receiving both ciphertexts, the evaluator can only succesfully decrypt the latter, which he does and sets g_label accordingly
    let e_label_received_from_ot = ot::decrypt(&real_pp, real_keypair.get_secret_key(), ciphertext_wj_1);
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

#[test] 
fn can_evaluate_xor_circuit() {
    let wire_gen = PointAndPermuteWires::new();
    let gate_gen = PointAndPermuteGates::new(wire_gen);
    let garbler = Garbler::new(gate_gen, wire_gen);

    let wi = WireBuild::new(0.to_biguint().unwrap()); // Need to create wires for the build, acting as input wires with output of layer 0.
    let wj = WireBuild::new(0.to_biguint().unwrap());
    // let circuit_build = circuit_builder::create_or(&wi, &wj);
    let circuit_build = vec![circuit_builder::create_gate(&wi, &wj, GateType::XOR)];
    let garbler_input_choices = vec![0 as u8]; // Garbler bit 0 as input. Assert somewhere we have just right amount of input choices
    // Garbler asks for as many key pairs as input gates. Amount of input gates should be stored somewhere? For now we know its only a or circuit.
    let pp = ot::PublicParameters::new();
    let keypair_real = ot::RealKeyPair::new(&pp);
    let pk_real = keypair_real.get_public_key();
    let sk_real = keypair_real.get_secret_key();
    let pk_oblivious = ot::ObliviousKeyPair::new(&pp).get_public_key();

    let evaluator_input_choices = vec![[(&pk_oblivious, &pp), (&pk_real, &pp)]]; // Eval has choosen to get bit 1
    let evaluator_decrypt_choices = vec![((&sk_real, &pp), 1 as u8)]; // chooses bit 1
    
    let (circuit, wi_inputs, wj_inputs, conversion_data) = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);

    let result = PointAndPermuteEvaluator::evaluate_circuit(&circuit, &wi_inputs, &wj_inputs, &conversion_data, &evaluator_decrypt_choices);
    assert!(result == 1)
}

#[test] 
fn can_evaluate_or_circuit() {
    let wire_gen = PointAndPermuteWires::new();
    let gate_gen = PointAndPermuteGates::new(wire_gen);
    let garbler = Garbler::new(gate_gen, wire_gen);

    let wi = WireBuild::new(0.to_biguint().unwrap()); // Need to create wires for the build, acting as input wires with output of layer 0.
    let wj = WireBuild::new(0.to_biguint().unwrap());
    // let circuit_build = circuit_builder::create_or(&wi, &wj);
    let circuit_build = circuit_builder::create_or(&wi, &wj);
    let garbler_input_choices = vec![0 as u8]; // Garbler bit 0 as input. Assert somewhere we have just right amount of input choices
    // Garbler asks for as many key pairs as input gates. Amount of input gates should be stored somewhere? For now we know its only a or circuit.
    let pp = ot::PublicParameters::new();
    let keypair_real = ot::RealKeyPair::new(&pp);
    let pk_real = keypair_real.get_public_key();
    let sk_real = keypair_real.get_secret_key();
    let pk_oblivious = ot::ObliviousKeyPair::new(&pp).get_public_key();

    let evaluator_input_choices = vec![[(&pk_oblivious, &pp), (&pk_real, &pp)]]; // Eval has choosen to get bit 1
    let evaluator_decrypt_choices = vec![((&sk_real, &pp), 1 as u8)]; // chooses bit 1
    
    let (circuit, wi_inputs, wj_inputs, conversion_data) = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);

    let result = PointAndPermuteEvaluator::evaluate_circuit(&circuit, &wi_inputs, &wj_inputs, &conversion_data, &evaluator_decrypt_choices);
    assert!(result == 1)
}