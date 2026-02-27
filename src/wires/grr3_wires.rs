use num_bigint::BigUint;
use rand::{thread_rng, Rng};
use crate::gates::gates::GateType;
use crate::wires::wires::{Wire, Wires};
use crate::crypto_utils::{gc_kdf_128, generate_label_lsb};

#[derive(Clone, Copy)]
pub struct GRR3Wires;

impl Wires for GRR3Wires {
    fn new() -> Self {
        Self
    }
    
    fn generate_input_wire(&self) -> Wire {
        let mut rng = thread_rng();
        let choice = rng.gen_bool(1.0 / 2.0);
        let w0 = generate_label_lsb(choice);
        let w1 = generate_label_lsb(!choice);
        Wire::new(w0, w1)
    }
    fn generate_output_wire(&self, wi: &Wire, wj: &Wire, gate: &GateType, gate_id: &BigUint) -> Wire {
        match gate {
            GateType::AND=>generate_and_wire(wi, wj, gate_id),
            GateType::XOR=>generate_xor_wire(wi, wj, gate_id),
        }
    }
}

fn generate_and_wire(wi: &Wire, wj: &Wire, gate_id: &BigUint) -> Wire {
    let w0c;
    let w1c;
    let w00 = get_00_wire(&wi, &wj, gate_id);
    if !wi.w1().bit(0) && !wj.w1().bit(0) {
        w0c = generate_label_lsb(!w00.bit(0));
        w1c = w00;
    } else {
        w1c = generate_label_lsb(!w00.bit(0));
        w0c = w00;
    }
    Wire::new(w0c, w1c)
}

fn generate_xor_wire(wi: &Wire, wj: &Wire, gate_id: &BigUint) -> Wire {
    let w0c;
    let w1c;
    let w00 = get_00_wire(&wi, &wj, gate_id);
    if (!wi.w0().bit(0) && !wj.w1().bit(0)) || (!wi.w1().bit(0) && !wj.w0().bit(0)) {
        w0c = generate_label_lsb(!w00.bit(0));
        w1c = w00;
    } else {
        w1c = generate_label_lsb(!w00.bit(0));
        w0c = w00;
    }
    Wire::new(w0c, w1c)
}

pub fn get_00_wire(wi: &Wire, wj: &Wire, gate_id: &BigUint) -> BigUint {
    for left in [&wi.w0(), &wi.w1()] {
        for right in [&wj.w0(), &wj.w1()] {
            if !left.bit(0) && !right.bit(0) {
                return gc_kdf_128(&left, &right, gate_id)
            }
        }
    }
    panic!("Couldn't find where both wires lsb was 0");
}