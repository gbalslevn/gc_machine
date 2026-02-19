use num_bigint::{BigUint};
use crate::crypto_utils::{gc_kdf_128, generate_label_lsb};
use crate::wires::original_wires::OriginalWires;
use crate::wires::point_and_permute_wires::PointAndPermuteWires;
use crate::wires::grr3_wires::{GRR3Wires, get_00_wire, generate_xor_wires, generate_and_wires};
use crate::wires::wires::Wires;

#[test]
fn test_generate_input_wires_returns_two_wires() {
    let wires = OriginalWires;
    let (w0, w1) = wires.generate_input_wires();
    assert!(w0.bits() <= 128);
    assert!(w1.bits() <= 128);
}

#[test]
fn test_generate_input_wires_randomness() {
    let wires = OriginalWires;
    let (w0_1, w1_1) = wires.generate_input_wires();
    let (w0_2, w1_2) = wires.generate_input_wires();

    // Very unlikely to generate same wires twice
    assert!(w0_1 != w0_2 || w1_1 != w1_2);
}

#[test]
fn test_generate_input_wires_opposite_lsb() {
    let wires = PointAndPermuteWires;
    let (w0, w1) = wires.generate_input_wires();

    // One should have LSB=0, the other LSB=1
    assert_ne!(w0.bit(0), w1.bit(0), "Wire LSBs should be opposite");
}

#[test]
fn test_get_00_wire_finds_correct_pair() {
    let w0i = generate_label_lsb(false); // LSB = 0
    let w1i = generate_label_lsb(true);  // LSB = 1
    let w0j = generate_label_lsb(true); // LSB = 1
    let w1j = generate_label_lsb(false);  // LSB = 0
    let gate_id = BigUint::from(1u32);

    let result = get_00_wire(&(w0i.clone(), w1i.clone()), &(w0j.clone(), w1j.clone()), &gate_id);

    // Should successfully find the (w0i, w0j) pair
    assert_eq!(result, gc_kdf_128(&w0i, &w1j, &gate_id));
}

#[test]
#[should_panic(expected = "Couldn't find where both wires lsb was 0")]
fn test_get_00_wire_panics_when_no_pair() {
    let w0i = generate_label_lsb(true);  // LSB = 1
    let w1i = generate_label_lsb(true);  // LSB = 1
    let w0j = generate_label_lsb(true);  // LSB = 1
    let w1j = generate_label_lsb(true);  // LSB = 1
    let gate_id = BigUint::from(1u32);

    get_00_wire(&(w0i, w1i), &(w0j, w1j), &gate_id);
}

#[test]
fn test_generate_and_wires_opposite_lsb() {
    let wires = GRR3Wires;
    let wi = wires.generate_input_wires();
    let wj = wires.generate_input_wires();
    let gate_id = BigUint::from(1u32);

    let (w0c, w1c) = generate_and_wires(&wi, &wj, &gate_id);

    assert_ne!(w0c.bit(0), w1c.bit(0), "Output wires should have opposite LSBs");
}

#[test]
fn test_generate_xor_wires_opposite_lsb() {
    let wires = GRR3Wires;
    let wi = wires.generate_input_wires();
    let wj = wires.generate_input_wires();
    let gate_id = BigUint::from(1u32);

    let (w0c, w1c) = generate_xor_wires(&wi, &wj, &gate_id);

    assert_ne!(w0c.bit(0), w1c.bit(0), "Output wires should have opposite LSBs");
}

#[test]
#[should_panic(expected = "Unknown gate")]
fn test_generate_output_wires_unknown_gate() {
    let wires = GRR3Wires;
    let wi = wires.generate_input_wires();
    let wj = wires.generate_input_wires();
    let gate_id = BigUint::from(1u32);

    wires.generate_output_wires(
        &wi, &wj, "or".to_string(), &gate_id
    );
}