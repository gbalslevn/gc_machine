use num_bigint::BigUint;

use crate::evaluator::evaluator::Evaluator;
use crate::evaluator::grr3_evaluator::GRR3Evaluator;
use crate::evaluator::original_evaluator::OriginalEvaluator;
use crate::evaluator::point_and_permute_evaluator::PointAndPermuteEvaluator;
use crate::gates::free_xor_gates::FreeXORGates;
use crate::evaluator::free_xor_evaluator::FreeXOREvaluator;
use crate::gates::gates::{GateType, Gates};
use crate::gates::grr3_gates::GRR3Gates;
use crate::gates::original_gates::OriginalGates;
use crate::gates::point_and_permute_gates::PointAndPermuteGates;
use crate::wires::free_xor_wires::FreeXORWires;
use crate::wires::grr3_wires::GRR3Wires;
use crate::wires::original_wires::OriginalWires;
use crate::wires::point_and_permute_wires::PointAndPermuteWires;
use crate::wires::wires::Wires;

#[test]
fn will_correctly_decrypt_xor_original() {
    let gate_type = GateType::XOR;
    let gate_id = BigUint::ZERO;
    let wire_gen = OriginalWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = OriginalGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj, gate_id);
    let mut evaluator = OriginalEvaluator::new();

    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gate_type, &gt.table);
    assert_eq!(&dec, gt.wo.w1())
}

#[test]
fn will_correctly_decrypt_and_original() {
    let gate_type = GateType::AND;
    let gate_id = BigUint::ZERO;
     let wire_gen = OriginalWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = OriginalGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj, gate_id);
    let mut evaluator = OriginalEvaluator::new();

    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gate_type, &gt.table);
    assert_eq!(&dec, gt.wo.w0())
}

#[test]
#[should_panic(expected = "No decryption with correct padding found!")]
fn will_panic_with_wrong_wires_original() {
    let gate_type = GateType::XOR;
    let gate_id = BigUint::ZERO;
    let wire_gen = OriginalWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = OriginalGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj, gate_id);
    let mut evaluator = OriginalEvaluator::new();

    // Evaluator has dummy wires
    let dummy_wires = gate_gen.wires.generate_input_wire();
    evaluator.evaluate_gate(&dummy_wires.w0(), &dummy_wires.w1(), &gt.gate_type, &gt.table);
}

#[test]
fn will_correctly_decrypt_xor_point_and_permute() {
    let gate_type = GateType::XOR;
    let gate_id = BigUint::ZERO;
    let wire_gen = PointAndPermuteWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = PointAndPermuteGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj, gate_id);
    let mut evaluator = PointAndPermuteEvaluator::new();


    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.table);
    assert_eq!(&dec, gt.wo.w1())
}

#[test]
fn will_correctly_decrypt_and_point_and_permute() {
    let gate_type = GateType::AND;
    let gate_id = BigUint::ZERO;
    let wire_gen = PointAndPermuteWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = PointAndPermuteGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj, gate_id);
    let mut evaluator = PointAndPermuteEvaluator::new();

    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.table);
    assert_eq!(&dec, gt.wo.w0())
}

#[test]
fn will_correctly_decrypt_xor_grr3() {
    let gate_type = GateType::XOR;
    let gate_id = BigUint::ZERO;
    let wire_gen = GRR3Wires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = GRR3Gates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj, gate_id);
    let mut evaluator = GRR3Evaluator::new();
    
    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.table);
    assert_eq!(&dec, gt.wo.w1())
}

#[test]
fn will_correctly_decrypt_and_grr3() {
    let gate_type = GateType::AND;
    let gate_id = BigUint::ZERO;
    let wire_gen = GRR3Wires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = GRR3Gates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj, gate_id);
    let mut evaluator = GRR3Evaluator::new();


    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.table);

    assert_eq!(&dec, gt.wo.w0())
}

#[test] 
fn xor_gate_for_free_xor_has_empty_table() {
    let gate_type = GateType::XOR;
    let gate_id = BigUint::ZERO;
    let wire_gen = FreeXORWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = FreeXORGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj, gate_id);
    assert!(gt.table.len() == 0);
}

#[test]
fn will_correctly_decrypt_xor_free_xor() {
    let gate_type = GateType::XOR;
    let gate_id = BigUint::ZERO;
    let wire_gen = FreeXORWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = FreeXORGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj, gate_id);
    let mut evaluator = FreeXOREvaluator::new();


    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.table);

    assert_eq!(&dec, gt.wo.w1())
}

#[test]
fn will_correctly_decrypt_and_free_xor() {
    let gate_type = GateType::AND;
    let gate_id = BigUint::ZERO;
    let wire_gen = FreeXORWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_gen = FreeXORGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj, gate_id);
    let mut evaluator = FreeXOREvaluator::new();
    
    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.table);

    assert_eq!(&dec, gt.wo.w0())
}