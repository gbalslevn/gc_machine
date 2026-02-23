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
use crate::wires::original_wires::OriginalWires;
use crate::wires::wires::Wires;

#[test]
fn will_correctly_decrypt_xor_original() {
    let gate_type = GateType::XOR;
    let gate_id = BigUint::ZERO;
    let gt = OriginalGates::new(gate_type, gate_id);

    // Evaluator has wi.0 and wj.1
    let dec = OriginalEvaluator::evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gate_type, &gt.gate_id, &gt.table);
    assert_eq!(&dec, gt.wo.w1())
}

#[test]
fn will_correctly_decrypt_and_original() {
    let gate_type = GateType::AND;
    let gate_id = BigUint::ZERO;
    let gt = OriginalGates::new(gate_type, gate_id);

    // Evaluator has wi.0 and wj.1
    let dec = OriginalEvaluator::evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gate_type, &gt.gate_id, &gt.table);
    assert_eq!(&dec, gt.wo.w0())
}

#[test]
#[should_panic(expected = "No decryption with correct padding found!")]
fn will_panic_with_wrong_wires_original() {
    let gate_type = GateType::XOR;
    let gate_id = BigUint::ZERO;
    let gt = OriginalGates::new(gate_type, gate_id);

    // Evaluator has dummy wires
    let dummy_wires = OriginalWires::generate_input_wire();
    OriginalEvaluator::evaluate_gate(&dummy_wires.w0(), &dummy_wires.w1(), &gt.gate_type, &gt.gate_id, &gt.table);
}

#[test]
fn will_correctly_decrypt_xor_point_and_permute() {
    let gate_type = GateType::XOR;
    let gate_id = BigUint::ZERO;
    let gt = PointAndPermuteGates::new(gate_type, gate_id);
    // Evaluator has wi.0 and wj.1
    let dec = PointAndPermuteEvaluator::evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.gate_id, &gt.table);
    assert_eq!(&dec, gt.wo.w1())
}

#[test]
fn will_correctly_decrypt_and_point_and_permute() {
    let gate_type = GateType::AND;
    let gate_id = BigUint::ZERO;
    let gt = PointAndPermuteGates::new(gate_type, gate_id);
    // Evaluator has wi.0 and wj.1
    let dec = PointAndPermuteEvaluator::evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.gate_id, &gt.table);
    assert_eq!(&dec, gt.wo.w0())
}

#[test]
fn will_correctly_decrypt_xor_grr3() {
    let gate_type = GateType::XOR;
    let gate_id = BigUint::ZERO;
    let gt = GRR3Gates::new(gate_type, gate_id);
    // Evaluator has wi.0 and wj.1
    let dec = GRR3Evaluator::evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.gate_id, &gt.table);

    assert_eq!(&dec, gt.wo.w1())
}

#[test]
fn will_correctly_decrypt_and_grr3() {
    let gate_type = GateType::AND;
    let gate_id = BigUint::ZERO;
    let gt = GRR3Gates::new(gate_type, gate_id);
    // Evaluator has wi.0 and wj.1
    let dec = GRR3Evaluator::evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.gate_id, &gt.table);

    assert_eq!(&dec, gt.wo.w0())
}

#[test] 
fn xor_gate_for_free_xor_has_empty_table() {
    let gate_type = GateType::XOR;
    let gate_id = BigUint::ZERO;
    let gt = FreeXORGates::new(gate_type, gate_id);
    assert!(gt.table.len() == 0);
}

#[test]
fn will_correctly_decrypt_xor_free_xor() {
    let gate_type = GateType::XOR;
    let gate_id = BigUint::ZERO;
    let gt = FreeXORGates::new(gate_type, gate_id);
    // Evaluator has wi.0 and wj.1
    let dec = FreeXOREvaluator::evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.gate_id, &gt.table);

    assert_eq!(&dec, gt.wo.w1())
}

#[test]
fn will_correctly_decrypt_and_free_xor() {
    let gate_type = GateType::AND;
    let gate_id = BigUint::ZERO;
    let gt = FreeXORGates::new(gate_type, gate_id);
    // Evaluator has wi.0 and wj.1
    let dec = FreeXOREvaluator::evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.gate_id, &gt.table);

    assert_eq!(&dec, gt.wo.w0())
}