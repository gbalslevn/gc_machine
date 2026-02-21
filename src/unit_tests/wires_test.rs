use num_bigint::{BigUint};
use crate::crypto_utils::{gc_kdf_128, generate_label_lsb};
use crate::gates::gates::GateType;
use crate::wires::original_wires::OriginalWires;
use crate::wires::point_and_permute_wires::PointAndPermuteWires;
use crate::wires::grr3_wires::{GRR3Wires, get_00_wire};
// use crate::wires::free_xor_wires::FreeXORWires;
use crate::wires::wires::Wires;

#[test]
fn test_generate_input_wires_returns_two_wires() {
    let wire = OriginalWires::generate_input_wire();
    assert!(wire.w0().bits() <= 128);
    assert!(wire.w1().bits() <= 128);
}

#[test]
fn test_generate_input_wires_randomness() {
    let w_0 = OriginalWires::generate_input_wire();
    let w_1 = OriginalWires::generate_input_wire();

    // Very unlikely to generate same wires twice
    assert!(w_0.w0() != w_1.w0() || w_0.w1() != w_1.w1());
}

#[test]
fn test_generate_input_wires_opposite_lsb() {
    let w = PointAndPermuteWires::generate_input_wire();

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

    let wi = GRR3Wires {w0: w0i, w1: w1i};
    let wj = GRR3Wires {w0: w0j, w1: w1j};

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

    let wi = GRR3Wires {w0: w0i, w1: w1i};
    let wj = GRR3Wires {w0: w0j, w1: w1j};
    
    GRR3Wires::generate_output_wire(&wi, &wj, &GateType::AND, &gate_id);
}

#[test]
fn test_generate_and_wires_opposite_lsb() {
    let wi = GRR3Wires::generate_input_wire();
    let wj = GRR3Wires::generate_input_wire();
    let gate_id = BigUint::from(1u32);

    let w = GRR3Wires::generate_output_wire(&wi, &wj, &GateType::AND, &gate_id);

    assert_ne!(w.w0().bit(0), w.w1().bit(0), "Output wires should have opposite LSBs");
}

#[test]
fn test_generate_xor_wires_opposite_lsb() {
    let wi = GRR3Wires::generate_input_wire();
    let wj = GRR3Wires::generate_input_wire();
    let gate_id = BigUint::from(1u32);

    let w = GRR3Wires::generate_output_wire(&wi, &wj, &GateType::XOR, &gate_id);

    assert_ne!(w.w0().bit(0), w.w1().bit(0), "Output wires should have opposite LSBs");
}

// #[test]
// fn are_output_wires_xor_of_input() {
//     let wires = FreeXORWires::new();
//     let wi = wires.generate_input_wires();
//     let wj = wires.generate_input_wires();
//     let gate_id = BigUint::from(1u32);
//     let wo = wires.generate_output_wires(&wi, &wj, "xor".to_string(), &gate_id);
//     assert_eq!(&wi.0 ^ &wj.0, wo.0);
//     assert_eq!(&wi.0 ^ &wj.1, wo.1);
//     assert_eq!(&wi.1 ^ &wj.0, wo.1);
//     assert_eq!(&wi.1 ^ &wj.1, wo.0);
// }

// #[test]
// fn are_and_wires_using_delta() {
//     let wires = FreeXORWires::new();
//     let delta = wires.delta();
//     let wi = wires.generate_input_wires();
//     let wj = wires.generate_input_wires();
//     let gate_id = BigUint::from(1u32);
//     let wo = wires.generate_output_wires(&wi, &wj, "and".to_string(), &gate_id);
//     assert_eq!(&wo.0 ^ delta, wo.1);
// }