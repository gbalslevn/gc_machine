use std::cmp::max;
use std::ops::Shr;
use gc_machine::circuit_builder::{CircuitBuilder};
use gc_machine::evaluator::evaluator::Evaluator;
use gc_machine::evaluator::original_evaluator::OriginalEvaluator;
use gc_machine::evaluator::point_and_permute_evaluator::PointAndPermuteEvaluator;
use gc_machine::garbler::Garbler;
use gc_machine::gates::point_and_permute_gates::PointAndPermuteGates;
use gc_machine::wires::point_and_permute_wires::PointAndPermuteWires;
use gc_machine::gates::gates::{GateType, Gates};
use gc_machine::gates::original_gates::OriginalGates;
use gc_machine::wires::original_wires::OriginalWires;
use gc_machine::{crypto_utils, websocket::{self, SocketConfig}};
use num_bigint::{BigUint, ToBigUint};
use gc_machine::ot::ot::{self, PublicParameters};
use gc_machine::ot::ot::encrypt;
use gc_machine::wires::wires::Wires;
use tokio_tungstenite::tungstenite::Message;

#[test]
// Garbler (with wire wi) and Evaluator(with wire wj) each provides a bit and can compare them using the standard yao garbled circuit.
fn can_compare_a_bit_using_std_yao() {

    // 1. Garbler creates circuit, a single XOR gate, and sends to evaluator
    let gate = GateType::XOR;
    let wire_gen = OriginalWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = OriginalGates::new(wire_gen);
    let current_index = gate_gen.get_index().clone();
    let xor_gate = gate_gen.generate_gate(gate, wi, wj);
    // 2. Evaluator receives circuit and chooses which bit-label he wants using OT.
    // 2.1 Evaluator prepares a ObliviousKeyPair and a RealKeyPar in that specific order, since he intends to receive the wirelabel for the 1-bit.
    let pp = ot::PublicParameters::new();
    let oblivious_keypair = ot::ObliviousKeyPair::new(&pp);
    let real_keypair = ot::RealKeyPair::new(&pp);
    // 2.2 The evaluator sends the publickey of both keypairs to the garbler who then encrypts wj.0 and wj.1 respectively.
    let _ciphertext_wj_0 = encrypt(&pp, &oblivious_keypair.get_public_key(), xor_gate.wj.w0());
    let ciphertext_wj_1 = encrypt(&pp, &real_keypair.get_public_key(), xor_gate.wj.w1());
    // 2.3 Upon receiving both ciphertexts, the evaluator can only succesfully decrypt the latter, which he does and sets g_label accordingly
    let e_label_received_from_ot = ot::decrypt(&pp, &real_keypair.get_secret_key(), &ciphertext_wj_1);
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
// Alice and Bob can connect to each other through a websocket and send 10 messages. 
async fn websocket_can_tx_and_rx_10_msg() {
    let socket_addr = "127.0.0.1:12346".to_string();
    let config = SocketConfig::new(socket_addr);
    let alice_socket_client = websocket::run(&config).await; 
    tokio::time::sleep(std::time::Duration::from_millis(50)).await; // Give server time to start
    let bob_socket_client = websocket::run(&config).await; 
    tokio::time::sleep(std::time::Duration::from_millis(50)).await; // Give server time to start
    
    // Alice sends 10 messages to Bob
    for i in 0..10 {
        alice_socket_client.send_message(Message::text(format!("{}", i))).await; // should implement error messages in socket
    }
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await; // Wait for messages to propogate
    assert_eq!(bob_socket_client.get_rx_msg_count().await, 10);
}

#[test] 
fn can_evaluate_or_circuit() {
    let wire_gen = PointAndPermuteWires::new();
    let gate_gen = PointAndPermuteGates::new(wire_gen);
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = PointAndPermuteEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    let input_wires = circuit_builder.build_input_wires(2);
    let _circuit_result = circuit_builder.build_or(&input_wires[0], &input_wires[1]);
    let circuit_build = circuit_builder.get_circuit_build();
    let garbler_input_choices = vec![0 as u8]; // Garbler bit 0 as input. Assert somewhere we have just right amount of input choices
    // Garbler asks for as many key pairs as input gates. Amount of input gates should be stored somewhere? For now we know its only a or circuit.
    let keypair_real = ot::RealKeyPair::new(&evaluator.get_pp());
    let pk_real = keypair_real.get_public_key();
    let sk_real = keypair_real.get_secret_key();
    let pk_oblivious = ot::ObliviousKeyPair::new(&evaluator.get_pp()).get_public_key();

    let evaluator_input_choices = vec![[pk_oblivious.clone(), pk_real.clone()]]; // Eval has choosen to get bit 1. Needs to send 2 times as he needs two 1 bits for the OR gate of input AND and XOR. Even though the OR gate abstracts it to seeing it as 1 bit. The input should be the same.
    let evaluator_decrypt_choices = vec![(sk_real.clone(), 1 as u8)]; // chooses bit 1
    
    let (circuit, wi_inputs, wj_inputs, conversion_data) = garbler.create_circuit(&circuit_build, &garbler_input_choices, evaluator_input_choices, evaluator.get_pp());

    let result = evaluator.evaluate_circuit(&circuit, &wi_inputs, &wj_inputs, evaluator_decrypt_choices, &conversion_data);
    assert!(result == 1)
}

#[test]
fn can_evaluate_xnor_circuit() {
    let wire_gen = PointAndPermuteWires::new();
    let gate_gen = PointAndPermuteGates::new(wire_gen);
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = PointAndPermuteEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    let input_wires = circuit_builder.build_input_wires(2);
    let _circuit_result = circuit_builder.build_xnor(&input_wires[0], &input_wires[1]);
    let circuit_build = circuit_builder.get_circuit_build();
    let garbler_input_choices = vec![0 as u8]; // Garbler bit 0 as input. Assert somewhere we have just right amount of input choices
    // Garbler asks for as many key pairs as input gates. Amount of input gates should be stored somewhere? For now we know its only a or circuit.
    let keypair_real = ot::RealKeyPair::new(&evaluator.get_pp());
    let pk_real = keypair_real.get_public_key();
    let sk_real = keypair_real.get_secret_key();
    let pk_oblivious = ot::ObliviousKeyPair::new(&evaluator.get_pp()).get_public_key();

    // Eval for when wi=0 and wj=1
    let evaluator_input_choices = vec![[pk_oblivious.clone(), pk_real.clone()]]; // Eval has choosen to get bit 1. 
    let evaluator_decrypt_choices = vec![(sk_real.clone(), 1 as u8)]; // chooses bit 1
    let (circuit, wi_inputs, wj_inputs, conversion_data) = garbler.create_circuit(&circuit_build, &garbler_input_choices, evaluator_input_choices, &evaluator.get_pp());
    let result = evaluator.evaluate_circuit(&circuit, &wi_inputs, &wj_inputs, evaluator_decrypt_choices, &conversion_data);
    
    assert!(result == 0);
    
    // Eval for when wi=0 and wj=0
    let evaluator_input_choices = vec![[pk_real.clone(), pk_oblivious.clone()]]; // Eval has choosen to get bit 0. 
    let evaluator_decrypt_choices = vec![(sk_real.clone(), 0 as u8)]; // chooses bit 0
    let (circuit, wi_inputs, wj_inputs, conversion_data) = garbler.create_circuit(&circuit_build, &garbler_input_choices, evaluator_input_choices, &evaluator.get_pp());
    let result = evaluator.evaluate_circuit(&circuit, &wi_inputs, &wj_inputs, evaluator_decrypt_choices, &conversion_data);

    assert!(result == 1);
}

// Tests for positive and negative case. Also for when inputs have unequal amount of bits. Should perhaps be split up at some point
#[test] 
fn can_evaluate_is_equal_circuit() {
    let wire_gen = OriginalWires::new();
    let gate_gen = OriginalGates::new(wire_gen);
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = OriginalEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    let a = 2.to_biguint().unwrap(); // bitlenght 2
    let b = 9.to_biguint().unwrap(); // bitlength 4
    // Comparing ints of unequal bitlength
    assert!(a != b);
    let bitlen_of_a = a.bits();
    let bitlen_of_b = b.bits();
    assert!(bitlen_of_a != bitlen_of_b);
    let required_bits = max(a.bits(), b.bits());

    circuit_builder.build_is_equal(required_bits);
    let circuit_build = circuit_builder.get_circuit_build();
    
    // Testing a=a
    let garbler_circuit_input_a = garbler.create_circuit_input(&a, required_bits); 
    let (evaluator_circuit_input_a, evaluator_decrypted_input_a) = evaluator.create_circuit_input(&a, required_bits);    
    let (circuit, wi_inputs, wj_inputs, conversion_data) = garbler.create_circuit(&circuit_build, &garbler_circuit_input_a, evaluator_circuit_input_a, evaluator.get_pp());

    let result = evaluator.evaluate_circuit(&circuit, &wi_inputs, &wj_inputs, evaluator_decrypted_input_a, &conversion_data);
    assert!(result == 1);
    
    // Testing a != b, eval holds b
    let (evaluator_circuit_input_b, evaluator_decrypted_input_b) = evaluator.create_circuit_input(&b, required_bits);    
    let (circuit, wi_inputs, wj_inputs, conversion_data) = garbler.create_circuit(&circuit_build, &garbler_circuit_input_a, evaluator_circuit_input_b, evaluator.get_pp());
    let result = evaluator.evaluate_circuit(&circuit, &wi_inputs, &wj_inputs, evaluator_decrypted_input_b, &conversion_data);
    assert!(result == 0);
}