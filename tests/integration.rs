use std::cmp::max;
use std::ops::{Shr};
use std::sync::Arc;
use std::time::Duration;
use gc_machine::circuit_builder::{CircuitBuilder};
use gc_machine::evaluator::evaluator::Evaluator;
use gc_machine::evaluator::free_xor_evaluator::FreeXOREvaluator;
use gc_machine::evaluator::grr3_evaluator::GRR3Evaluator;
use gc_machine::evaluator::half_gates_evaluator::HalfGatesEvaluator;
use gc_machine::evaluator::original_evaluator::OriginalEvaluator;
use gc_machine::evaluator::point_and_permute_evaluator::PointAndPermuteEvaluator;
use gc_machine::garbler::{Garbler};
use gc_machine::gates::free_xor_gate_gen::FreeXORGateGen;
use gc_machine::gates::grr3_gate_gen::GRR3GateGen;
use gc_machine::gates::half_gates_gate_gen::HalfGatesGateGen;
use gc_machine::gates::point_and_permute_gate_gen::PointAndPermuteGateGen;
use gc_machine::ot::eg_elliptic::{self};
use gc_machine::peer::{Peer};
use gc_machine::websocket::Response;
use gc_machine::gates::gate_gen::{GateType, GateGen};
use gc_machine::gates::original_gate_gen::OriginalGateGen;
use gc_machine::wires::original_wire_gen::OriginalWireGen;
use gc_machine::{crypto_utils};
use num_bigint::{BigUint, ToBigUint};
use gc_machine::wires::wire_gen::WireGen;

#[test]
// Garbler (with wire wi) and Evaluator(with wire wj) each provides a bit and can compare them using the standard yao garbled circuit.
fn can_compare_a_bit_using_std_yao() {

    // 1. Garbler creates circuit, a single XOR gate, and sends to evaluator
    let gate = GateType::XOR;
    let mut wire_gen = OriginalWireGen::new();
    let mut rng = wire_gen.get_rng().clone();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = OriginalGateGen::new();
    let current_index = gate_gen.get_index().clone();
    let xor_gate = gate_gen.generate_gate(gate, wi, wj);
    // 2. Evaluator receives circuit and chooses which bit-label he wants using OT.
    // 2.1 Evaluator prepares a ObliviousKeyPair and a RealKeyPar in that specific order, since he intends to receive the wirelabel for the 1-bit.
    let oblivious_keypair = eg_elliptic::ObliviousKeyPair::new();
    let real_keypair = eg_elliptic::RealKeyPair::new();
    // 2.2 The evaluator sends the publickey of both keypairs to the garbler who then encrypts wj.0 and wj.1 respectively.
    let _ciphertext_wj_0 = eg_elliptic::encrypt(&mut rng, &oblivious_keypair.get_pk(), xor_gate.wj.w0());
    let ciphertext_wj_1 = eg_elliptic::encrypt(&mut rng, &real_keypair.get_pk(), xor_gate.wj.w1());
    // 2.3 Upon receiving both ciphertexts, the evaluator can only succesfully decrypt the latter, which he does and sets g_label accordingly
    let e_label_received_from_ot = eg_elliptic::decrypt( &real_keypair.get_sk(), &ciphertext_wj_1);
    // 3. Garbler sends her bit as a label (g_label) as well as the evaluators labels. Evaluator now has what is needed to evaluate.
    let g_label = xor_gate.wi.w0();
    let e_label = e_label_received_from_ot;
    let mut decrypted_output_label =  BigUint::ZERO;
    let key = crypto_utils::gc_kdf(&g_label, &e_label, &current_index);
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

#[tokio::test]
async fn can_eval_circuit_over_socket() {
    // Create two peers which connects to each other
    let gate_gen = OriginalGateGen::new();
    let evaluator = OriginalEvaluator::new();
    let evaluator_peer = get_peer(gate_gen.clone(), evaluator.clone(), false).await;
    let garbler_peer = get_peer(gate_gen, evaluator, false).await;

    garbler_peer.connect(evaluator_peer.get_address()).await.expect("Could not connect to evaluator_peer");
    tokio::time::sleep(Duration::from_secs(1)).await; // Wait for it to connect

    // Create a circuit build which both peers in some way has agreed on
    let garbler_input = 12.to_biguint().unwrap();
    let evaluator_input = 12.to_biguint().unwrap();
    let required_bits = max(&garbler_input, &evaluator_input).bits(); // They somehow know the max amount of bits needed 
    let mut builder = CircuitBuilder::new();
    let (garbler_wires, evaluator_wires) = builder.set_input_wires(required_bits);
    builder.build_is_equal(&garbler_wires, &evaluator_wires);
    let cb = builder.get_circuit_build();
    
    // They both prepare to start the protocol
    garbler_peer.setup_circuit_context(garbler_input, cb.clone(), required_bits).await;
    evaluator_peer.setup_circuit_context(evaluator_input, cb, required_bits).await;

    let response = garbler_peer.execute_protocol(evaluator_peer.get_peer_id()).await.expect("Execute protocol failed");
    if let Response::GCResult(result) = response {
        assert_eq!(result, 1);
    }
}

#[test]
fn can_evaluate_is_equal() {
    let gate_gen = OriginalGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut evaluator = OriginalEvaluator::new();

    // true case
    evaluate_is_equal(1.to_biguint().unwrap(), 1.to_biguint().unwrap(), true, &mut garbler, &mut evaluator);
    
    // false case
    evaluate_is_equal(3.to_biguint().unwrap(), 5.to_biguint().unwrap(), false, &mut garbler, &mut evaluator);
}

// Tests for positive and negative case. Also for when inputs have unequal amount of bits. Should perhaps be split up at some point
#[test] 
fn can_evaluate_is_equal_circuit_for_all_optimisations() {
    let gate_gen = OriginalGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut evaluator = OriginalEvaluator::new();
    evaluate_is_equal(100.to_biguint().unwrap(), 100.to_biguint().unwrap(), true, &mut garbler, &mut evaluator);

    let gate_gen = PointAndPermuteGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut evaluator = PointAndPermuteEvaluator::new();
    evaluate_is_equal(100.to_biguint().unwrap(), 100.to_biguint().unwrap(), true, &mut garbler, &mut evaluator);

    let gate_gen = GRR3GateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut evaluator = GRR3Evaluator::new();
    evaluate_is_equal(100.to_biguint().unwrap(), 100.to_biguint().unwrap(), true, &mut garbler, &mut evaluator);

    let gate_gen = FreeXORGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut evaluator = FreeXOREvaluator::new();
    evaluate_is_equal(100.to_biguint().unwrap(), 100.to_biguint().unwrap(), true, &mut garbler, &mut evaluator);

    let gate_gen = HalfGatesGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut evaluator = HalfGatesEvaluator::new();
    evaluate_is_equal(100.to_biguint().unwrap(), 100.to_biguint().unwrap(), true, &mut garbler, &mut evaluator);
}

#[test] 
fn can_evaulate_naive_if_circuit() {
    let gate_gen = OriginalGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut evaluator = OriginalEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    // Create circuit build which from a function computes true if garblers and evaluators inputs are equal. Else it returns false. 
    let required_bits = 6; //  Enable working with numbers up to 64
    let (garbler_wires, evaluator_wires) = circuit_builder.set_input_wires(required_bits);
    let is_equal = circuit_builder.build_is_equal(&garbler_wires, &evaluator_wires); 
    let true_block = circuit_builder.build_and(&is_equal, &is_equal); // 1 AND 1 = 1
    let false_block = circuit_builder.build_and(&is_equal, &is_equal); // 0 AND 0 = 0
    circuit_builder.build_if(&is_equal, &true_block.output, &false_block.output);
    let circuit_build = circuit_builder.get_circuit_build();
    
    // **** Evaluate for true case ****
    let a = 32.to_biguint().unwrap();
    let b = 32.to_biguint().unwrap();
    let garbler_input_choices = garbler.create_circuit_input(&a, required_bits);
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&b, required_bits);
    // Garbler create circuit
    let circuit = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);
    // Evaluator evaluates circuit. We expect true to return as a = b
    let result = evaluator.evaluate_circuit(&circuit_build, circuit, &evaluator_decrypt_values);
    assert_eq!(result, true as u32);
    
    // **** Evaluate for false case ****
    let c = 15.to_biguint().unwrap();
    let d = 32.to_biguint().unwrap();
    let garbler_input_choices = garbler.create_circuit_input(&c, required_bits);
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&d, required_bits);
    // Garbler create circuit
    let circuit = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);
    // Evaluator evaluates circuit. We expect false to return as c != d
    let result = evaluator.evaluate_circuit(&circuit_build, circuit, &evaluator_decrypt_values);
    assert_eq!(result, false as u32) 
}

#[test]
fn can_evaluate_stacked_if_circuit() {
    let gate_gen = HalfGatesGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut evaluator = HalfGatesEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    // Create circuit build which from a function computes true if garblers and evaluators inputs are equal. Else it returns false. 
    let required_bits = 6; //  Enable working with numbers up to 64
    let (garbler_wires, evaluator_wires) = circuit_builder.set_input_wires(required_bits);
    let is_equal = circuit_builder.build_is_equal(&garbler_wires, &evaluator_wires); // is_equal is true as 32==32
    let mut true_block = circuit_builder.build_and(&is_equal, &is_equal); // 1 AND 1 = 1 
    let mut false_block = circuit_builder.build_and(&is_equal, &is_equal); // 0 AND 0 = 0
    
    circuit_builder.build_stacked_if(&is_equal, &mut true_block, &mut false_block);
    let circuit_build = circuit_builder.get_circuit_build();

    // **** Evaluate for true case ****
    let a = 32.to_biguint().unwrap();
    let b = 32.to_biguint().unwrap();
    let garbler_input_choices = garbler.create_circuit_input(&a, required_bits);
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&b, required_bits);
    // Garbler create circuit
    let circuit = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);
    // Evaluator evaluates circuit. We expect true to return as a = b
    let result = evaluator.evaluate_circuit(&circuit_build, circuit, &evaluator_decrypt_values);
    assert_eq!(result, true as u32);
    
    // **** Evaluate for false case ****
    let c = 15.to_biguint().unwrap();
    let d = 32.to_biguint().unwrap();
    let garbler_input_choices = garbler.create_circuit_input(&c, required_bits);
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&d, required_bits);
    // Garbler create circuit
    let circuit = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);
    // Evaluator evaluates circuit. We expect false to return as c != d
    let result = evaluator.evaluate_circuit(&circuit_build, circuit, &evaluator_decrypt_values);
    assert_eq!(result, false as u32) 
}

#[test]
fn can_evaluate_stacked_if_with_adder_and_mul_circuit() { 
    let gate_gen = HalfGatesGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut evaluator = HalfGatesEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    // Create circuit build which from a function computes true if garblers and evaluators inputs are equal. Else it returns false. 
    // If true return garbler_input * evaluator_input, else return garbler_input+evaluator_input
    let required_bits = 7; //  Enable working with numbers up to 64
    let (garbler_wires, evaluator_wires) = circuit_builder.set_input_wires(required_bits);
    let is_equal = circuit_builder.build_is_equal(&garbler_wires, &evaluator_wires);
    let mut garbl_plus_eval = circuit_builder.build_adder(&garbler_wires, &evaluator_wires);
    let mut garbl_times_eval = circuit_builder.build_multiplier(&garbler_wires, &evaluator_wires);
    
    circuit_builder.build_stacked_if(&is_equal, &mut garbl_plus_eval, &mut garbl_times_eval);
    let circuit_build = circuit_builder.get_circuit_build();

    // **** Evaluate for true case ****
    let a = 21.to_biguint().unwrap();
    let b = 21.to_biguint().unwrap();
    let garbler_input_choices = garbler.create_circuit_input(&a, required_bits);
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&b, required_bits);
    let circuit = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);
    let result = evaluator.evaluate_circuit(&circuit_build, circuit, &evaluator_decrypt_values);
    assert_eq!(result.to_biguint().unwrap(), a*b);
    
    // **** Evaluate for false case ****
    let c = 80.to_biguint().unwrap();
    let d = 74.to_biguint().unwrap();
    let garbler_input_choices = garbler.create_circuit_input(&c, required_bits);
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&d, required_bits);
    let circuit = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);
    let result = evaluator.evaluate_circuit(&circuit_build, circuit, &evaluator_decrypt_values);
    assert_eq!(result.to_biguint().unwrap(), c+d) 
}

#[test]
fn can_evaluate_nested_stacked_if() {
    let gate_gen = HalfGatesGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut evaluator = HalfGatesEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    let required_bits = 7;
    let (garbler_wires, evaluator_wires) = circuit_builder.set_input_wires(required_bits);
    let is_equal = circuit_builder.build_is_equal(&garbler_wires, &evaluator_wires);
    let mut adder_0 = circuit_builder.build_adder(&garbler_wires, &evaluator_wires);
    let mut adder_1 = circuit_builder.build_adder(&garbler_wires, &garbler_wires);
    
    let mut nested_if = circuit_builder.build_stacked_if(&is_equal, &mut adder_0, &mut adder_1);
    circuit_builder.build_stacked_if(&is_equal, &mut nested_if, &mut adder_1);
    let circuit_build = circuit_builder.get_circuit_build();

    // **** Evaluate for true case ****
    let a = 18.to_biguint().unwrap();
    let b = 18.to_biguint().unwrap();
    let garbler_input_choices = garbler.create_circuit_input(&a, required_bits);
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&b, required_bits);
    let circuit = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);
    let result = evaluator.evaluate_circuit(&circuit_build, circuit, &evaluator_decrypt_values);
    assert_eq!(result.to_biguint().unwrap(), a+b); 
    
    // **** Evaluate for false case ****
    let c = 100.to_biguint().unwrap();
    let d = 2.to_biguint().unwrap();
    let garbler_input_choices = garbler.create_circuit_input(&c, required_bits);
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&d, required_bits);
    let circuit = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);
    let result = evaluator.evaluate_circuit(&circuit_build, circuit, &evaluator_decrypt_values);
    assert_eq!(result.to_biguint().unwrap(), c.clone()+d) 
}

#[track_caller]
fn evaluate_is_equal<G, E>(a : BigUint, b : BigUint, expected_result : bool, garbler : &mut Garbler<G>, evaluator : &mut E) where G: GateGen, E: Evaluator, {
    // Garbler's and Evaluator's input
    let required_bits = max(a.bits(), b.bits());
    let mut circuit_builder = CircuitBuilder::new();
    let (garbler_wires, evaluator_wires) = circuit_builder.set_input_wires(required_bits);

    let garbler_input_choices = garbler.create_circuit_input(&a, required_bits);
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&b, required_bits);

    // Create circuit build
    circuit_builder.build_is_equal(&garbler_wires, &evaluator_wires);
    let circuit_build = circuit_builder.get_circuit_build();

    // Garbler garbles and evaluator evaluates
    let circuit = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);
    let result = evaluator.evaluate_circuit(&circuit_build, circuit, &evaluator_decrypt_values);

    assert_eq!(result, expected_result as u32);
}

#[test]
fn evaluate_adder() {
    let gate_gen = OriginalGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut evaluator = OriginalEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    let a = 123.to_biguint().unwrap();
    let b = 45678.to_biguint().unwrap();
    let required_bits = max(a.bits(), b.bits());

    let (input_wires_garbler, input_wires_evaluator) = circuit_builder.set_input_wires(required_bits);

    circuit_builder.build_adder(&input_wires_garbler, &input_wires_evaluator);
    let circuit_build = circuit_builder.get_circuit_build();

    // Garbler input
    let garbler_input_choices = garbler.create_circuit_input(&a, required_bits);

    // Evaluator input
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&b, required_bits);

    // Garble circuit
    let circuit = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);

    // Evaluate circuit
    let result = evaluator.evaluate_circuit(&circuit_build, circuit, &evaluator_decrypt_values);

    assert_eq!(result, 45801);
}

#[test]
fn evaluate_multiplier() {
    let gate_gen = OriginalGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut evaluator = OriginalEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    let a = 1234.to_biguint().unwrap();
    let b = 1234.to_biguint().unwrap();
    let required_bits = max(a.bits(), b.bits());

    let (garbler_wires, evaluator_wires) = circuit_builder.set_input_wires(required_bits);

    circuit_builder.build_multiplier(&garbler_wires, &evaluator_wires);
    let circuit_build = circuit_builder.get_circuit_build();

    // Garbler input
    let garbler_input_choices = garbler.create_circuit_input(&a, required_bits);

    // Evaluator input
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&b, required_bits);

    // Garble circuit
    let circuit = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);

    // Evaluate circuit
    let result = evaluator.evaluate_circuit(&circuit_build, circuit, &evaluator_decrypt_values);

    assert_eq!(result, 1522756);

}


// Maybe this test shows we do not need to provide equal amount of bits as input. If we do not then this test can be simplified. Should we use a constant wire for padding? Or another solution. 
#[test]
fn can_add_numbers_of_unequal_bitlength() {
    let gate_gen = OriginalGateGen::new();
    let mut garbler = Garbler::new(gate_gen);
    let mut evaluator = OriginalEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    let one_bit_number = 0.to_biguint().unwrap(); 
    let two_bit_number = 2.to_biguint().unwrap(); 
    let required_bits = max(one_bit_number.bits(), two_bit_number.bits());

    let (input_wires_garbler, input_wires_evaluator) = circuit_builder.set_input_wires(required_bits);

    // Garbler input, holds the one bit number
    let garbler_input_choices = garbler.create_circuit_input(&one_bit_number, required_bits);

    // Evaluator input, holds the two bit number
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&two_bit_number, required_bits);

    let adder_block = circuit_builder.build_adder(&input_wires_garbler, &input_wires_garbler); // garbler holds 0, 0+0 = 0, 1 bit required
    circuit_builder.build_adder(&adder_block.output, &input_wires_evaluator); // 0+2 = 2, 2 bits required
    let circuit_build = circuit_builder.get_circuit_build();

    // Garble circuit
    let circuit = garbler.create_circuit(&circuit_build, &garbler_input_choices, &evaluator_input_choices);

    // Evaluate circuit
    let result = evaluator.evaluate_circuit(&circuit_build, circuit, &evaluator_decrypt_values);

    assert_eq!(result.to_biguint().unwrap(), two_bit_number);
}

async fn get_peer<G, E>(gate_gen : G, evaluator : E, with_logging : bool) -> Arc<Peer<G, E>> where
    G: GateGen + Send + Sync + 'static,
    E: Evaluator + Send + Sync + 'static, {
    let garbler = Garbler::new(gate_gen);
    let peer = Peer::new(garbler, evaluator).await;
    if with_logging {
        let _ = peer.start_logging().await;
    }
    peer
}