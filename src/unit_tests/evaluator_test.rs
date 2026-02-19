use num_bigint::ToBigUint;
use crate::wires::wires::Wires;
use crate::gates::gates::Gates;
use crate::evaluator::evaluator::Evaluator;

use crate::wires::original_wires::OriginalWires;
use crate::gates::original_gates::OriginalGates;
use crate::evaluator::original_evaluator::OriginalEvaluator;

use crate::wires::point_and_permute_wires::PointAndPermuteWires;
use crate::gates::point_and_permute_gates::PointAndPermuteGates;
use crate::evaluator::point_and_permute_evaluator::PointAndPermuteEvaluator;

use crate::wires::grr3_wires::GRR3Wires;
use crate::gates::grr3_gates::GRR3Gates;
use crate::evaluator::grr3_evaluator::GRR3Evaluator;

#[test]
fn will_correctly_decrypt_xor_original() {
    let wires = OriginalWires;
    let wi = wires.generate_input_wires();
    let wj = wires.generate_input_wires();
    let gate = "xor";
    let gate_id = 0.to_biguint().unwrap();
    let wo = wires.generate_output_wires(&wi, &wj, gate.to_string(), &gate_id);
    let tt = OriginalGates::get_tt(&wi, &wj, &wo, gate.to_string());
    let gc = OriginalGates::get_garbled_gate(&tt, &gate_id);
    // Evaluator has wi.0 and wj.1
    let dec = OriginalEvaluator::evaluate_gate(&wi.0, &wj.1, &gate_id, gate.to_string(), &gc);
    assert_eq!(dec, wo.1)
}

#[test]
fn will_correctly_decrypt_and_original() {
    let wires = OriginalWires;
    let wi = wires.generate_input_wires();
    let wj = wires.generate_input_wires();
    let gate = "and";
    let gate_id = 0.to_biguint().unwrap();
    let wo = wires.generate_output_wires(&wi, &wj, gate.to_string(), &gate_id);
    let tt = OriginalGates::get_tt(&wi, &wj, &wo, gate.to_string());
    let gc = OriginalGates::get_garbled_gate(&tt, &gate_id);
    // Evaluator has wi.0 and wj.1
    let dec = OriginalEvaluator::evaluate_gate(&wi.0, &wj.1, &gate_id, gate.to_string(), &gc);
    assert_eq!(dec, wo.0)
}

#[test]
#[should_panic(expected = "No output with correct padding found!")]
fn will_panic_with_wrong_wires_original() {
    let wires = OriginalWires;
    let wi = wires.generate_input_wires();
    let wj = wires.generate_input_wires();
    let gate_type = "xor";
    let gate_id = 0.to_biguint().unwrap();
    let wo = wires.generate_output_wires(&wi, &wj, gate_type.to_string(), &gate_id);
    let tt = OriginalGates::get_xor_tt(&wi, &wj, &wo);
    let gc = OriginalGates::get_garbled_gate(&tt, &gate_id);
    // Evaluator has dummy wires
    let dummy_wires = wires.generate_input_wires();
    OriginalEvaluator::evaluate_gate(&dummy_wires.0, &dummy_wires.1, &gate_id, gate_type.to_string(), &gc);
}

#[test]
fn will_correctly_decrypt_xor_point_and_permute() {
    let wires = PointAndPermuteWires;
    let wi = wires.generate_input_wires();
    let wj = wires.generate_input_wires();
    let gate = "xor";
    let gate_id = 0.to_biguint().unwrap();
    let wo = wires.generate_output_wires(&wi, &wj, gate.to_string(), &gate_id);
    let tt = PointAndPermuteGates::get_tt(&wi, &wj, &wo, gate.to_string());
    let gc = PointAndPermuteGates::get_garbled_gate(&tt, &gate_id);
    // Evaluator has wi.0 and wj.1
    let dec = PointAndPermuteEvaluator::evaluate_gate(&wi.0, &wj.1, &gate_id, gate.to_string(), &gc);
    assert_eq!(dec, wo.1)
}

#[test]
fn will_correctly_decrypt_and_point_and_permute() {
    let wires = PointAndPermuteWires;
    let wi = wires.generate_input_wires();
    let wj = wires.generate_input_wires();
    let gate = "and";
    let gate_id = 0.to_biguint().unwrap();
    let wo = wires.generate_output_wires(&wi, &wj, gate.to_string(), &gate_id);
    let tt = PointAndPermuteGates::get_tt(&wi, &wj, &wo, gate.to_string());
    let gc = PointAndPermuteGates::get_garbled_gate(&tt, &gate_id);
    // Evaluator has wi.0 and wj.1
    let dec = PointAndPermuteEvaluator::evaluate_gate(&wi.0, &wj.1, &gate_id, gate.to_string(), &gc);
    assert_eq!(dec, wo.0)
}

#[test]
fn will_correctly_decrypt_xor_grr3() {
    let wires = GRR3Wires;
    let wi = wires.generate_input_wires();
    let wj = wires.generate_input_wires();
    let gate = "xor";
    let gate_id = 0.to_biguint().unwrap();
    let wo = wires.generate_output_wires(&wi, &wj, gate.to_string(), &gate_id);
    let tt = GRR3Gates::get_tt(&wi, &wj, &wo, gate.to_string());
    let gc = GRR3Gates::get_garbled_gate(&tt, &gate_id);
    // Evaluator has wi.0 and wj.1
    let dec = GRR3Evaluator::evaluate_gate(&wi.0, &wj.1, &gate_id, gate.to_string(), &gc);
    assert_eq!(dec, wo.1)
}

#[test]
fn will_correctly_decrypt_and_grr3() {
    let wires = GRR3Wires;
    let wi = wires.generate_input_wires();
    let wj = wires.generate_input_wires();
    let gate = "and";
    let gate_id = 0.to_biguint().unwrap();
    let wo = wires.generate_output_wires(&wi, &wj, gate.to_string(), &gate_id);
    let tt = GRR3Gates::get_tt(&wi, &wj, &wo, gate.to_string());
    let gc = GRR3Gates::get_garbled_gate(&tt, &gate_id);
    // Evaluator has wi.0 and wj.1
    let dec = GRR3Evaluator::evaluate_gate(&wi.0, &wj.1, &gate_id, gate.to_string(), &gc);
    assert_eq!(dec, wo.0)
}