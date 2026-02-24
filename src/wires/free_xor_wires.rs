use num_bigint::{BigUint};
use crate::gates::gates::GateType;
use crate::wires::wires::{Wire, Wires};
use crate::crypto_utils::{gc_kdf_128, generate_label_lsb, generate_label};

pub struct FreeXORWires {
    pub delta: BigUint,
}

impl FreeXORWires {
    pub fn delta(&self) -> &BigUint {
        &self.delta // Why does each each wire need to hold delta? Perhaps the gate should and we make a standard wire struct for all gates. Even better the garbler should hold it. 
    }
}

impl Wires for FreeXORWires {
    fn new() -> Self {
        let delta = generate_label_lsb(true); // to ensure point and permute holds
        FreeXORWires { delta }
    }

    fn generate_input_wire(&self) -> Wire {
        let delta = &self.delta;
        let w0 = generate_label();
        let w1 = &w0 ^ delta;
        Wire::new(w0, w1)
    }
    fn generate_output_wire(wi: &Wire, wj: &Wire, gate: &GateType, gate_id: &BigUint) -> Wire {
        let delta = wi.w0() ^ wi.w1(); // We derive delta, kind of a hack
        match gate {
            GateType::AND=>generate_and_wires(delta, &wi, &wj, gate_id),
            GateType::XOR=>generate_xor_wires(delta, &wi, &wj, gate_id),
        }
    }
}

pub fn generate_and_wires(delta: BigUint, wi: &Wire, wj: &Wire, gate_id: &BigUint) -> Wire {
    let w0c;
    let w1c;
    let w00 = get_00_wire(&wi, &wj, gate_id);
    if !wi.w1().bit(0) && !wj.w1().bit(0) {
        w0c = &w00 ^ delta.clone();
        w1c = w00;
    } else {
        w1c = &w00 ^ delta.clone();
        w0c = w00;
    }
    Wire::new(w0c, w1c)
}

pub fn generate_xor_wires(delta: BigUint, wi: &Wire, wj: &Wire, _gate_id: &BigUint) -> Wire {
    let w0c = wi.w0() ^ wj.w0();
    let w1c = &w0c ^ delta.clone();
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