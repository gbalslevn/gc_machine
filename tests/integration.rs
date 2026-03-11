use std::cmp::max;
use std::ops::Shr;
use gc_machine::circuit_builder::{CircuitBuilder};
use gc_machine::evaluator::evaluator::Evaluator;
use gc_machine::evaluator::original_evaluator::OriginalEvaluator;
use gc_machine::evaluator::point_and_permute_evaluator::PointAndPermuteEvaluator;
use gc_machine::garbler::Garbler;
use gc_machine::gates::point_and_permute_gate_gen::PointAndPermuteGateGen;
use gc_machine::ot::eg_elliptic::{self, ObliviousKeyPair, RealKeyPair};
use gc_machine::wires::point_and_permute_wire_gen::PointAndPermuteWireGen;
use gc_machine::gates::gate_gen::{GateType, GateGen};
use gc_machine::gates::original_gate_gen::OriginalGateGen;
use gc_machine::wires::original_wire_gen::OriginalWireGen;
use gc_machine::{crypto_utils, websocket::{self, SocketConfig}};
use num_bigint::{BigUint, ToBigUint};
use gc_machine::wires::wire_gen::WireGen;
use tokio_tungstenite::tungstenite::Message;

#[test]
// Garbler (with wire wi) and Evaluator(with wire wj) each provides a bit and can compare them using the standard yao garbled circuit.
fn can_compare_a_bit_using_std_yao() {

    // 1. Garbler creates circuit, a single XOR gate, and sends to evaluator
    let gate = GateType::XOR;
    let mut wire_gen = OriginalWireGen::new();
    let mut rng = wire_gen.get_rng().clone();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = OriginalGateGen::new(wire_gen);
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
fn can_evaluate_xor_circuit() {
    // Initialization
    let wire_gen = PointAndPermuteWireGen::new();
    let gate_gen = PointAndPermuteGateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = PointAndPermuteEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    // Create circuit recipe using circuit builder
    let input_wires = circuit_builder.build_input_wires(2u32);
    circuit_builder.build_xor(&input_wires[0], &input_wires[1]);
    let circuit_build = circuit_builder.get_circuit_build();

    // Create Garbler input. This is just what wire to choose (0)
    let garbler_input_choices = vec![1 as u8];

    // Create Evaluator Input. This involves creating a fake and real public key. And stating what order they should be used in
    let real_key = eg_elliptic::RealKeyPair::new();
    let oblivious_key = eg_elliptic::ObliviousKeyPair::new();
    let evaluator_input_choices = vec![[real_key.get_pk().clone(), oblivious_key.get_pk().clone()]];
    let evaluator_decrypt_choices = vec![(real_key.get_sk().clone(), 0u8), (real_key.get_sk().clone(), 0u8)];

    // Garbler garbles
    let (circuit_eval, garbler_input, evaluator_input, conversion_table) =garbler.create_circuit(&circuit_build, &garbler_input_choices, evaluator_input_choices);

    // Evaluator evaluates
    let result = evaluator.evaluate_circuit(&circuit_eval, &garbler_input, &evaluator_input, evaluator_decrypt_choices, &conversion_table);
    assert_eq!(result, 1);
}

#[test]
fn can_evaluate_and_circuit() {
    // Initialization
    let wire_gen = PointAndPermuteWireGen::new();
    let gate_gen = PointAndPermuteGateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = PointAndPermuteEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    // Create circuit recipe using circuit builder
    let input_wires = circuit_builder.build_input_wires(2u32);
    circuit_builder.build_and(&input_wires[0], &input_wires[1]);
    let circuit_build = circuit_builder.get_circuit_build();

    // Create Garbler input. This is just what wire to choose (1)
    let garbler_input_choices = vec![1 as u8];

    // Create Evaluator Input. This involves creating a fake and real public key. And stating what order they should be used in
    let real_key = eg_elliptic::RealKeyPair::new();
    let oblivious_key = eg_elliptic::ObliviousKeyPair::new();
    let evaluator_input_choices = vec![[oblivious_key.get_pk().clone(), real_key.get_pk().clone()]];
    let evaluator_decrypt_choices = vec![(real_key.get_sk().clone(), 1u8)];

    // Garbler garbles
    let (circuit_eval, garbler_input, evaluator_input, conversion_table) =garbler.create_circuit(&circuit_build, &garbler_input_choices, evaluator_input_choices);

    // Evaluator evaluates
    let result = evaluator.evaluate_circuit(&circuit_eval, &garbler_input, &evaluator_input, evaluator_decrypt_choices, &conversion_table);
    assert_eq!(result, 1);
}

#[test]
fn can_evaluate_three_gates_circuit() {
    // Initialization
    let wire_gen = PointAndPermuteWireGen::new();
    let gate_gen = PointAndPermuteGateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = PointAndPermuteEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    // Create circuit recipe using circuit builder
    let input_wires = circuit_builder.build_input_wires(4u32);
    let and_output_1 = circuit_builder.build_and(&input_wires[0], &input_wires[2]);
    let and_output_2 = circuit_builder.build_and(&input_wires[1], &input_wires[3]);
    circuit_builder.build_and(&and_output_1, &and_output_2);
    let circuit_build = circuit_builder.get_circuit_build();
    // Create Garbler input. This is just what wire to choose (0)
    let garbler_input_choices = vec![1 as u8, 1 as u8];

    // Create Evaluator Input. This involves creating a fake and real public key. And stating what order they should be used in
    let real_key = eg_elliptic::RealKeyPair::new();
    let oblivious_key = eg_elliptic::ObliviousKeyPair::new();
    let evaluator_input_choices = vec![[oblivious_key.get_pk().clone(), real_key.get_pk().clone()], [oblivious_key.get_pk().clone(), real_key.get_pk().clone()]];
    let evaluator_decrypt_choices = vec![(real_key.get_sk().clone(), 1u8), (real_key.get_sk().clone(), 1u8)];

    // Garbler garbles
    let (circuit_eval, garbler_input, evaluator_input, conversion_table) =garbler.create_circuit(&circuit_build, &garbler_input_choices, evaluator_input_choices);
    // Evaluator evaluates
    let result = evaluator.evaluate_circuit(&circuit_eval, &garbler_input, &evaluator_input, evaluator_decrypt_choices, &conversion_table);
    assert_eq!(result, 1);
}

#[test] 
fn can_evaluate_or_circuit() {
    let wire_gen = PointAndPermuteWireGen::new();
    let gate_gen = PointAndPermuteGateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = PointAndPermuteEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    let input_wires = circuit_builder.build_input_wires(2);
    let _circuit_result = circuit_builder.build_or(&input_wires[0], &input_wires[1]);
    let circuit_build = circuit_builder.get_circuit_build();
    // Garbler asks for as many key pairs as input gates. Amount of input gates should be stored somewhere? For now, we know it's only a or circuit.
    let garbler_input_choices = vec![0 as u8]; // Garbler bit 0 as input. Assert somewhere we have just right amount of input choices
    // Garbler asks for as many key pairs as input gates. Amount of input gates should be stored somewhere? For now we know its only a or circuit.
    let keypair_real = RealKeyPair::new();
    let pk_real = keypair_real.get_pk();
    let sk_real = keypair_real.get_sk();
    let keypair_oblivious = ObliviousKeyPair::new();
    let pk_oblivious = keypair_oblivious.get_pk();
    
    let evaluator_input_choices = vec![[pk_oblivious.clone(), pk_real.clone()]]; // Eval has choosen to get bit 1. Needs to send 2 times as he needs two 1 bits for the OR gate of input AND and XOR. Even though the OR gate abstracts it to seeing it as 1 bit. The input should be the same.
    let evaluator_decrypt_choices = vec![(sk_real.clone(), 1 as u8)]; // chooses bit 1

    let (circuit, garbler_input, evaluator_input, conversion_data) = garbler.create_circuit(&circuit_build, &garbler_input_choices, evaluator_input_choices);

    let result = evaluator.evaluate_circuit(&circuit, &garbler_input, &evaluator_input, evaluator_decrypt_choices, &conversion_data);
    assert_eq!(result, 1)
}

#[test]
fn can_evaluate_xnor_circuit() {
    let wire_gen = PointAndPermuteWireGen::new();
    let gate_gen = PointAndPermuteGateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = PointAndPermuteEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    let input_wires = circuit_builder.build_input_wires(2);
    let _circuit_result = circuit_builder.build_xnor(&input_wires[0], &input_wires[1]);
    let circuit_build = circuit_builder.get_circuit_build();
    let garbler_input_choices = vec![0u8]; // Garbler bit 0 as input. Assert somewhere we have just right amount of input choices
    // Garbler asks for as many key pairs as input gates. Amount of input gates should be stored somewhere? For now, we know it's only a or circuit.
    let keypair_real = RealKeyPair::new();
    let pk_real = keypair_real.get_pk();
    let sk_real = keypair_real.get_sk();
    let keypair_oblivious = ObliviousKeyPair::new();
    let pk_oblivious = keypair_oblivious.get_pk();

    // Eval for when wi=0 and wj=1
    let evaluator_input_choices = vec![[pk_oblivious.clone(), pk_real.clone()]]; // Eval has choosen to get bit 1. 
    let evaluator_decrypt_choices = vec![(sk_real.clone(), 1u8)]; // chooses bit 1
    let (circuit, wi_inputs, wj_inputs, conversion_data) = garbler.create_circuit(&circuit_build, &garbler_input_choices, evaluator_input_choices);
    let result = evaluator.evaluate_circuit(&circuit, &wi_inputs, &wj_inputs, evaluator_decrypt_choices, &conversion_data);
    
    assert_eq!(result, 0);
    
    // Eval for when wi=0 and wj=0
    let evaluator_input_choices = vec![[pk_real.clone(), pk_oblivious.clone()]]; // Eval has choosen to get bit 0. 
    let evaluator_decrypt_choices = vec![(sk_real.clone(), 0u8)]; // chooses bit 0
    let (circuit, wi_inputs, wj_inputs, conversion_data) = garbler.create_circuit(&circuit_build, &garbler_input_choices, evaluator_input_choices);
    let result = evaluator.evaluate_circuit(&circuit, &wi_inputs, &wj_inputs, evaluator_decrypt_choices, &conversion_data);

    assert_eq!(result, 1);
}

// Tests for positive and negative case. Also for when inputs have unequal amount of bits. Should perhaps be split up at some point
#[test] 
fn can_evaluate_is_equal_circuit() {
    let wire_gen = OriginalWireGen::new();
    let gate_gen = OriginalGateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = OriginalEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    // First half of the wires are the garblers
    let input_wires = circuit_builder.build_input_wires(4);

    circuit_builder.build_is_equal(input_wires);
    let circuit_build = circuit_builder.get_circuit_build();

    // Garbler's and Evaluator's input
    let a = 3.to_biguint().unwrap();
    let b = 3.to_biguint().unwrap();
    let required_bits = max(a.bits(), b.bits());
    let garbler_input_choices = garbler.create_circuit_input(&a, required_bits);
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&b, required_bits);

    // Garbler create circuit
    let (circuit, garbler_input, evaluator_input, conversion_table) = garbler.create_circuit(&circuit_build, &garbler_input_choices, evaluator_input_choices);
    let result = evaluator.evaluate_circuit(&circuit, &garbler_input, &evaluator_input, evaluator_decrypt_values, &conversion_table);
    // Testing a=a



    assert_eq!(result, 1);
}