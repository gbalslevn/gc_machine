use crate::evaluator::evaluator::Evaluator;
use crate::evaluator::grr3_evaluator::GRR3Evaluator;
use crate::evaluator::original_evaluator::OriginalEvaluator;
use crate::evaluator::point_and_permute_evaluator::PointAndPermuteEvaluator;
use crate::gates::free_xor_gates::FreeXORGates;
use crate::evaluator::free_xor_evaluator::FreeXOREvaluator;
use crate::evaluator::half_gates_evaluator::HalfGatesEvaluator;
use crate::gates::gates::{GateType, Gates};
use crate::gates::grr3_gates::GRR3Gates;
use crate::gates::half_gates_gates::HalfGatesGates;
use crate::gates::original_gates::OriginalGates;
use crate::gates::point_and_permute_gates::PointAndPermuteGates;
use crate::wires::free_xor_wires::FreeXORWires;
use crate::wires::grr3_wires::GRR3Wires;
use crate::wires::half_gates_wires::HalfGateWires;
use crate::wires::original_wires::OriginalWires;
use crate::wires::point_and_permute_wires::PointAndPermuteWires;
use crate::wires::wires::Wires;

#[test]
fn will_correctly_decrypt_xor_original() {
    let gate_type = GateType::XOR;
    let wire_gen = OriginalWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = OriginalGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj);
    let mut evaluator = OriginalEvaluator::new();

    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gate_type, &gt.table);
    assert_eq!(&dec, gt.wo.w1())
}

#[test]
fn will_correctly_decrypt_and_original() {
    let gate_type = GateType::AND;
     let wire_gen = OriginalWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = OriginalGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj);
    let mut evaluator = OriginalEvaluator::new();

    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gate_type, &gt.table);
    assert_eq!(&dec, gt.wo.w0())
}

#[test]
#[should_panic(expected = "No decryption with correct padding found!")]
fn will_panic_with_wrong_wires_original() {
    let gate_type = GateType::XOR;
    let wire_gen = OriginalWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = OriginalGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj);
    let mut evaluator = OriginalEvaluator::new();

    // Evaluator has dummy wires
    let dummy_wires = gate_gen.wires.generate_input_wire();
    evaluator.evaluate_gate(&dummy_wires.w0(), &dummy_wires.w1(), &gt.gate_type, &gt.table);
}

#[test]
fn will_correctly_decrypt_xor_point_and_permute() {
    let gate_type = GateType::XOR;
    let wire_gen = PointAndPermuteWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = PointAndPermuteGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj);
    let mut evaluator = PointAndPermuteEvaluator::new();


    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.table);
    assert_eq!(&dec, gt.wo.w1())
}

#[test]
fn will_correctly_decrypt_and_point_and_permute() {
    let gate_type = GateType::AND;
    let wire_gen = PointAndPermuteWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = PointAndPermuteGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj);
    let mut evaluator = PointAndPermuteEvaluator::new();

    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.table);
    assert_eq!(&dec, gt.wo.w0())
}

#[test]
fn will_correctly_decrypt_xor_grr3() {
    let gate_type = GateType::XOR;
    let wire_gen = GRR3Wires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = GRR3Gates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj);
    let mut evaluator = GRR3Evaluator::new();
    
    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.table);
    assert_eq!(&dec, gt.wo.w1())
}

#[test]
fn will_correctly_decrypt_and_grr3() {
    let gate_type = GateType::AND;
    let wire_gen = GRR3Wires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = GRR3Gates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj);
    let mut evaluator = GRR3Evaluator::new();


    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.table);

    assert_eq!(&dec, gt.wo.w0())
}

#[test] 
fn xor_gate_for_free_xor_has_empty_table() {
    let gate_type = GateType::XOR;
    let wire_gen = FreeXORWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = FreeXORGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj);
    assert!(gt.table.len() == 0);
}

#[test]
fn will_correctly_decrypt_xor_free_xor() {
    let gate_type = GateType::XOR;
    let wire_gen = FreeXORWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = FreeXORGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj);
    let mut evaluator = FreeXOREvaluator::new();


    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.table);

    assert_eq!(&dec, gt.wo.w1())
}

#[test]
fn will_correctly_decrypt_and_free_xor() {
    let gate_type = GateType::AND;
    let wire_gen = FreeXORWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = FreeXORGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj);
    let mut evaluator = FreeXOREvaluator::new();
    
    // Evaluator has wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.table);

    assert_eq!(&dec, gt.wo.w0())
}

#[test]
fn will_correctly_decrypt_and_half_gates() {
    let gate_type = GateType::AND;
    let wire_gen = HalfGateWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let mut gate_gen = HalfGatesGates::new(wire_gen);
    let gt = gate_gen.generate_gate(gate_type, wi, wj);
    let mut evaluator = HalfGatesEvaluator::new();

    // Evaluator holds wi.0 and wj.1
    let dec = evaluator.evaluate_gate(&gt.wi.w0(), &gt.wj.w1(), &gt.gate_type, &gt.table);
    assert_eq!(&dec, gt.wo.w0())
}

#[test]
fn can_decrypt_multiple_gates_with_internal_index() {
    let gate_type = GateType::AND;
    let wire_gen = HalfGateWires::new();
    let wi1 = wire_gen.generate_input_wire();
    let wj1 = wire_gen.generate_input_wire();
    let wi2 = wire_gen.generate_input_wire();
    let wj2 = wire_gen.generate_input_wire();
    let mut gate_gen = HalfGatesGates::new(wire_gen);
    let gt1 = gate_gen.generate_gate(gate_type, wi1, wj1);
    let gt2 = gate_gen.generate_gate(gate_type, wi2, wj2);
    let gt3 = gate_gen.generate_gate(gate_type, gt1.wo, gt2.wo);


    let mut evaluator = HalfGatesEvaluator::new();

    // Evaluator holds wi1.0 and wj1.1 and gets wo.0
    let dec1 = evaluator.evaluate_gate(&gt1.wi.w0(), &gt1.wj.w1(), &gt1.gate_type, &gt1.table);
    assert_eq!(&dec1, gt3.wi.w0());
    // Evaluator holds wi2.1 and wj2.1 and gets wo.1
    let dec2 = evaluator.evaluate_gate(&gt2.wi.w1(), &gt2.wj.w1(), &gt2.gate_type, &gt2.table);
    assert_eq!(&dec2, gt3.wj.w1());
    // Evaluator has output wires 0 and 1 and will get 0 as the result.
    let dec3 = evaluator.evaluate_gate(&dec1, &dec2, &gt3.gate_type, &gt3.table);

    assert_eq!(&dec3, gt3.wo.w0());
    assert_eq!(gate_gen.get_index(), evaluator.get_index());
}

#[test]
fn cannot_decrypt_multiple_gates_with_wrong_order() {
    let gate_type = GateType::AND;
    let wire_gen = HalfGateWires::new();
    let wi1 = wire_gen.generate_input_wire();
    let wj1 = wire_gen.generate_input_wire();
    let wi2 = wire_gen.generate_input_wire();
    let wj2 = wire_gen.generate_input_wire();
    let mut gate_gen = HalfGatesGates::new(wire_gen);
    let gt1 = gate_gen.generate_gate(gate_type, wi1, wj1);
    let gt2 = gate_gen.generate_gate(gate_type, wi2, wj2);
    let gt3 = gate_gen.generate_gate(gate_type, gt1.wo, gt2.wo);


    let mut evaluator = HalfGatesEvaluator::new();

    // Decryption in wrong order will not work
    // Evaluator holds wi2.1 and wj2.1 and gets wo.1
    let dec2 = evaluator.evaluate_gate(&gt2.wi.w1(), &gt2.wj.w1(), &gt2.gate_type, &gt2.table);
    // Evaluator holds wi1.0 and wj1.1 and gets wo.0
    let dec1 = evaluator.evaluate_gate(&gt1.wi.w0(), &gt1.wj.w1(), &gt1.gate_type, &gt1.table);

    // Evaluator has output wires 0 and 1 and will get 0 as the result.
    let dec3 = evaluator.evaluate_gate(&dec1, &dec2, &gt3.gate_type, &gt3.table);
    assert_ne!(&dec3, gt3.wo.w0());
}