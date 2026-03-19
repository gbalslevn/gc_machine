use std::cmp::max;
use std::ops::Shr;
use std::thread::sleep;
use std::time::Duration;
use gc_machine::circuit_builder::{CircuitBuilder};
use gc_machine::evaluator::evaluator::Evaluator;
use gc_machine::evaluator::free_xor_evaluator::FreeXOREvaluator;
use gc_machine::evaluator::grr3_evaluator::GRR3Evaluator;
use gc_machine::evaluator::half_gates_evaluator::HalfGatesEvaluator;
use gc_machine::evaluator::original_evaluator::OriginalEvaluator;
use gc_machine::evaluator::point_and_permute_evaluator::PointAndPermuteEvaluator;
use gc_machine::garbler::Garbler;
use gc_machine::gates::free_xor_gate_gen::FreeXORGateGen;
use gc_machine::gates::grr3_gate_gen::GRR3GateGen;
use gc_machine::gates::half_gates_gate_gen::HalfGatesGateGen;
use gc_machine::gates::point_and_permute_gate_gen::PointAndPermuteGateGen;
use gc_machine::ot::eg_elliptic::{self};
use gc_machine::wires::free_xor_wire_gen::FreeXORWireGen;
use gc_machine::wires::grr3_wire_gen::GRR3WireGen;
use gc_machine::wires::half_gates_wire_gen::HalfGatesWireGen;
use gc_machine::wires::point_and_permute_wire_gen::PointAndPermuteWireGen;
use gc_machine::gates::gate_gen::{GateType, GateGen};
use gc_machine::gates::original_gate_gen::OriginalGateGen;
use gc_machine::wires::original_wire_gen::OriginalWireGen;
use gc_machine::{crypto_utils, websocket::{self, SocketConfig}};
use num_bigint::{BigUint, ToBigUint};
use gc_machine::wires::wire_gen::WireGen;
use tokio::time::timeout;
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

    // wait for at most 10 sec
    let result = timeout(Duration::from_secs(10), async { 
        while bob_socket_client.get_rx_msg_count().await < 10 {
            sleep(Duration::from_millis(50)); // check every 50 ms if msg_count < 10
        }
    }).await;

    assert!(result.is_ok());
}


#[test]
fn can_evaluate_is_equal() {
    let wire_gen = OriginalWireGen::new();
    let gate_gen = OriginalGateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = OriginalEvaluator::new();

    // true case
    evaluate_is_equal(1.to_biguint().unwrap(), 1.to_biguint().unwrap(), true, &mut garbler, &mut evaluator);
    
    // false case
    evaluate_is_equal(3.to_biguint().unwrap(), 5.to_biguint().unwrap(), false, &mut garbler, &mut evaluator);
}

// Tests for positive and negative case. Also for when inputs have unequal amount of bits. Should perhaps be split up at some point
#[test] 
fn can_evaluate_is_equal_circuit_for_all_optimisations() {
    let wire_gen = OriginalWireGen::new();
    let gate_gen = OriginalGateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = OriginalEvaluator::new();
    evaluate_is_equal(100.to_biguint().unwrap(), 100.to_biguint().unwrap(), true, &mut garbler, &mut evaluator);

    let wire_gen = PointAndPermuteWireGen::new();
    let gate_gen = PointAndPermuteGateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = PointAndPermuteEvaluator::new();
    evaluate_is_equal(100.to_biguint().unwrap(), 100.to_biguint().unwrap(), true, &mut garbler, &mut evaluator);

    let wire_gen = GRR3WireGen::new();
    let gate_gen = GRR3GateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = GRR3Evaluator::new();
    evaluate_is_equal(100.to_biguint().unwrap(), 100.to_biguint().unwrap(), true, &mut garbler, &mut evaluator);

    let wire_gen = FreeXORWireGen::new();
    let gate_gen = FreeXORGateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = FreeXOREvaluator::new();
    evaluate_is_equal(100.to_biguint().unwrap(), 100.to_biguint().unwrap(), true, &mut garbler, &mut evaluator);

    let wire_gen = HalfGatesWireGen::new();
    let gate_gen = HalfGatesGateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = HalfGatesEvaluator::new();
    evaluate_is_equal(100.to_biguint().unwrap(), 100.to_biguint().unwrap(), true, &mut garbler, &mut evaluator);
}

#[test] 
fn can_evaulate_if_circuit() {
    let wire_gen = OriginalWireGen::new();
    let gate_gen = OriginalGateGen::new(wire_gen.clone());
    let mut garbler = Garbler::new(gate_gen, wire_gen);
    let mut evaluator = OriginalEvaluator::new();
    let mut circuit_builder = CircuitBuilder::new();

    // Garbler's and Evaluator's input
    let a = 32.to_biguint().unwrap();
    let b = 32.to_biguint().unwrap();
    let required_bits = max(a.bits(), b.bits());
    let input_wires = circuit_builder.build_input_wires((required_bits * 2) as u32);

    let garbler_input_choices = garbler.create_circuit_input(&a, required_bits);
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&b, required_bits);

    // Create circuit build
    let is_equal = circuit_builder.build_is_equal(input_wires);
    let true_case = circuit_builder.build_and_output(&is_equal, &is_equal); // 1 AND 1 = 1
    let false_case = circuit_builder.build_and_output(&is_equal, &is_equal); // 0 AND 0 = 0
    circuit_builder.build_if(&is_equal, &true_case, &false_case);
    let circuit_build = circuit_builder.get_circuit_build();
    
    // Garbler create circuit
    let (garbled_gates, constant_wires, garbler_input, evaluator_input, conversion_table) = garbler.create_circuit(&circuit_build, &garbler_input_choices, evaluator_input_choices);

    // Checks the return of the if statement
    let result = evaluator.evaluate_circuit(&circuit_build, &garbled_gates, &constant_wires, &garbler_input, &evaluator_input, evaluator_decrypt_values, conversion_table);
    assert_eq!(result, 1)
}

#[track_caller]
fn evaluate_is_equal<G, W, E>(a : BigUint, b : BigUint, expected_result : bool, garbler : &mut Garbler<G, W>, evaluator : &mut E) where G: GateGen<W>, W: WireGen, E: Evaluator, {
    // Garbler's and Evaluator's input
    let required_bits = max(a.bits(), b.bits());
    let mut circuit_builder = CircuitBuilder::new();
    let input_wires = circuit_builder.build_input_wires((required_bits * 2) as u32);

    let garbler_input_choices = garbler.create_circuit_input(&a, required_bits);
    let (evaluator_input_choices, evaluator_decrypt_values) = evaluator.create_circuit_input(&b, required_bits);

    // Create circuit build
    circuit_builder.build_is_equal(input_wires);
    let circuit_build = circuit_builder.get_circuit_build();
    println!("Output wires in circuitBuild: {:#?}", circuit_build.output_wires);

    // Garbler create circuit
    let (garbled_gates, constant_wires, garbler_input, evaluator_input, conversion_table) = garbler.create_circuit(&circuit_build, &garbler_input_choices, evaluator_input_choices);
    let result = evaluator.evaluate_circuit(&circuit_build, &garbled_gates, &constant_wires, &garbler_input, &evaluator_input, evaluator_decrypt_values, conversion_table);
    // Testing a=a

    assert_eq!(result, expected_result as u32);
}
