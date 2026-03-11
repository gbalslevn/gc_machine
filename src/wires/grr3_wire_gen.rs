use std::ops::Add;

use num_bigint::BigUint;
use rand_chacha::ChaCha20Rng;
use crate::gates::gate_gen::GateType;
use crate::wires::wire_gen::{Wire, WireGen};
use crate::crypto_utils::{self, gc_kdf_128, generate_label_lsb};

#[derive(Clone)]
pub struct GRR3WireGen {
    rng : ChaCha20Rng
}

impl WireGen for GRR3WireGen {
    fn new() -> Self {
        let rng = crypto_utils::gen_rng();
        Self { rng }
    }
    
    fn generate_input_wire(&mut self) -> Wire {
        let choice = crypto_utils::gen_bool(&mut self.rng);
        let w0 = generate_label_lsb(&mut self.rng, choice);
        let w1 = generate_label_lsb(&mut self.rng,!choice);
        Wire::new(w0, w1)
    }
    fn generate_output_wire(&mut self, wi: &Wire, wj: &Wire, gate: &GateType, gate_id: &BigUint) -> Wire {
        match gate {
            GateType::AND=>generate_and_wire(&mut self.rng, wi, wj, gate_id),
            GateType::XOR=>generate_xor_wire(&mut self.rng, wi, wj, gate_id),
        }
    }
    fn get_rng(&self) -> &ChaCha20Rng {
        &self.rng
    }
    fn new_rng(&mut self) {
        self.rng = crypto_utils::gen_rng()
    }
}

fn generate_and_wire(rng : &mut ChaCha20Rng, wi: &Wire, wj: &Wire, gate_id: &BigUint) -> Wire {
    let w0c;
    let w1c;
    let w00 = get_00_wire(&wi, &wj, gate_id);
    if !wi.w1().bit(0) && !wj.w1().bit(0) {
        w0c = generate_label_lsb(rng,!w00.bit(0));
        w1c = w00;
    } else {
        w1c = generate_label_lsb(rng,!w00.bit(0));
        w0c = w00;
    }
    Wire::new(w0c, w1c)
}

fn generate_xor_wire(rng : &mut ChaCha20Rng, wi: &Wire, wj: &Wire, gate_id: &BigUint) -> Wire {
    let w0c;
    let w1c;
    let w00 = get_00_wire(&wi, &wj, gate_id);
    if (!wi.w0().bit(0) && !wj.w1().bit(0)) || (!wi.w1().bit(0) && !wj.w0().bit(0)) {
        w0c = generate_label_lsb(rng,!w00.bit(0));
        w1c = w00;
    } else {
        w1c = generate_label_lsb(rng,!w00.bit(0));
        w0c = w00;
    }
    Wire::new(w0c, w1c)
}

pub fn get_00_wire(wi: &Wire, wj: &Wire, gate_id: &BigUint) -> BigUint {
    for left in [&wi.w0(), &wi.w1()] {
        for right in [&wj.w0(), &wj.w1()] {
            if !left.bit(0) && !right.bit(0) {
                let mn = crypto_utils::get_magic_number();
                return gc_kdf_128(&left.add(mn), right, gate_id)
            }
        }
    }
    panic!("Couldn't find where both wires lsb was 0");
}