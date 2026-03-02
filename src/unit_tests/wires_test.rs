use num_bigint::{BigUint};
use crate::crypto_utils::{gc_kdf_128, gc_kdf_hg, generate_label_lsb};
use crate::gates::gates::GateType;
use crate::wires::free_xor_wires::FreeXORWires;
use crate::wires::original_wires::OriginalWires;
use crate::wires::point_and_permute_wires::PointAndPermuteWires;
use crate::wires::grr3_wires::{GRR3Wires, get_00_wire};
use crate::wires::half_gates_wires::HalfGateWires;
// use crate::wires::free_xor_wires::FreeXORWires;
use crate::wires::wires::{Wire, Wires};

#[test]
fn test_generate_input_wires_returns_two_wires() {
    let wire_gen = OriginalWires::new();
    let w = wire_gen.generate_input_wire();
    assert!(w.w0().bits() <= 128);
    assert!(w.w1().bits() <= 128);
}

#[test]
fn test_generate_input_wires_randomness() {
    let wire_gen = OriginalWires::new();
    let w_0 = wire_gen.generate_input_wire();
    let w_1 = wire_gen.generate_input_wire();

    // Very unlikely to generate same wires twice
    assert!(w_0.w0() != w_1.w0() || w_0.w1() != w_1.w1());
}

#[test]
fn test_generate_input_wires_opposite_lsb() {
    let wire_gen = PointAndPermuteWires::new();
    let w = wire_gen.generate_input_wire();

    // One should have LSB=0, the other LSB=1
    assert_ne!(w.w0().bit(0), w.w1().bit(0), "Wire LSBs should be opposite");
}

#[test]
fn test_get_00_wire_finds_correct_pair() {
    let w0i = generate_label_lsb(false); // LSB = 0
    let w1i = generate_label_lsb(true);  // LSB = 1
    let w0j = generate_label_lsb(true); // LSB = 1
    let w1j = generate_label_lsb(false);  // LSB = 0
    let gate_id = BigUint::from(1u32);

    let wi = Wire::new(w0i, w1i);
    let wj = Wire::new(w0j, w1j);

    let result = get_00_wire(&wi, &wj, &gate_id);

    // Should successfully find the (w0i, w0j) pair
    assert_eq!(result, gc_kdf_128(&wi.w0(), &wj.w1(), &gate_id));
}

#[test]
#[should_panic(expected = "Couldn't find where both wires lsb was 0")]
fn test_get_00_wire_panics_when_no_pair() {
    let w0i = generate_label_lsb(true);  // LSB = 1
    let w1i = generate_label_lsb(true);  // LSB = 1
    let w0j = generate_label_lsb(true);  // LSB = 1
    let w1j = generate_label_lsb(true);  // LSB = 1
    let gate_id = BigUint::from(1u32);

    let wi = Wire::new(w0i, w1i);
    let wj = Wire::new(w0j, w1j);
    
    let wire_gen = GRR3Wires::new();
    wire_gen.generate_output_wire(&wi, &wj, &GateType::AND, &gate_id);
}

#[test]
fn test_generate_grr3_and_wires_opposite_lsb() {
    let wire_gen = GRR3Wires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_id = BigUint::from(1u32);

    let w = wire_gen.generate_output_wire(&wi, &wj, &GateType::AND, &gate_id);

    assert_ne!(w.w0().bit(0), w.w1().bit(0), "Output wires should have opposite LSBs");
}

#[test]
fn test_generate_xor_wires_opposite_lsb() {
    let wire_gen = GRR3Wires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_id = BigUint::from(1u32);

    let w = wire_gen.generate_output_wire(&wi, &wj, &GateType::XOR, &gate_id);

    assert_ne!(w.w0().bit(0), w.w1().bit(0), "Output wires should have opposite LSBs");
}

#[test]
fn are_output_wires_xor_of_input() {
    let wire_gen = FreeXORWires::new();
    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_id = BigUint::from(1u32);
    let wo = wire_gen.generate_output_wire(&wi, &wj, &GateType::XOR, &gate_id);
    assert_eq!(&(wi.w0() ^ wj.w0()), wo.w0());
    assert_eq!(&(wi.w0() ^ wj.w1()), wo.w1());
    assert_eq!(&(wi.w1() ^ wj.w0()), wo.w1());
    assert_eq!(&(wi.w1() ^ wj.w1()), wo.w0());
}

#[test]
fn are_and_wires_using_delta() {
    let wire_gen = FreeXORWires::new();
    let delta = wire_gen.delta();

    let wi = wire_gen.generate_input_wire();
    let wj = wire_gen.generate_input_wire();
    let gate_id = BigUint::from(1u32);
    let wo = wire_gen.generate_output_wire(&wi, &wj, &GateType::AND, &gate_id);
    assert_eq!(&(wo.w0() ^ delta), wo.w1());
}

#[test]
fn do_lsb_determine_output_wires() {
    let wire_gen = HalfGateWires::new();
    let delta = wire_gen.delta();
    let gate = GateType::AND;
    let gate_id = BigUint::from(0u32);
    let next_gate_id = BigUint::from(1u32);

    let wi0 = generate_label_lsb(true);
    let wi1 = &wi0 ^ delta;
    let wi = Wire::new(wi0, wi1);
    let wj0 = generate_label_lsb(false);
    let wj1 = &wj0 ^ delta;
    let wj = Wire::new(wj0, wj1);
    let wo = wire_gen.generate_output_wire(&wi, &wj, &gate, &gate_id);

    let wg0 = gc_kdf_hg(&wi.w0(), &gate_id) ^ gc_kdf_hg(&wi.w1(), &gate_id) ^gc_kdf_hg(&wi.w0(), &gate_id);
    let we0 = gc_kdf_hg(&wj.w0(), &next_gate_id);
    let w0 = wg0 ^ we0;
    assert_eq!(wo.w0(), &w0);
    assert_eq!(wo.w1(), &(w0 ^ delta));
}