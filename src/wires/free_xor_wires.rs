use std::str::FromStr;
use num_bigint::{BigUint, ToBigUint};
use crate::gates::gates::GateType;
use crate::wires::wires::Wires;
use crate::crypto_utils::{gc_kdf_128, generate_label_lsb, generate_label};

pub struct FreeXORWires {
    pub w0: BigUint,
    pub w1: BigUint,
    pub delta: BigUint,
}

impl FreeXORWires {
    pub fn delta(&self) -> &BigUint {
        &self.delta // Why does each each wire need to hold delta? Perhaps the gate should and we make a standard wire struct for all gates. Even better the garbler should hold it. 
    }
}

impl Wires for FreeXORWires {
    fn w0(&self) -> &BigUint {
        &self.w0
    }

    fn w1(&self) -> &BigUint {
        &self.w1
    }

    fn generate_input_wire() -> Self {
        // let delta = generate_label_lsb(true); // to ensure point and permute holds
        let delta_str = "44118055070050376567495178382802105751"; // WARNING THIS SHOULD BE CREATED FRESH FOR EACH CIRCUIT
        let delta = BigUint::from_str(delta_str).expect("Invalid number format");
        println!("delta is : {}", delta);
        let w0 = generate_label();
        let w1 = &w0 ^ &delta;
        Self {w0: w0, w1: w1, delta : delta}
    }
    fn generate_output_wire(wi: &Self, wj: &Self, gate: &GateType, gate_id: &BigUint) -> Self {
        let delta = &wi.w0 ^ &wi.w1; // We derive delta, kind of a hack
        match gate {
            GateType::AND=>generate_and_wires(delta, &wi, &wj, gate_id),
            GateType::XOR=>generate_xor_wires(delta, &wi, &wj, gate_id),
        }
    }
}

pub fn generate_and_wires(delta: BigUint, wi: &FreeXORWires, wj: &FreeXORWires, gate_id: &BigUint) -> FreeXORWires {
    let w0c;
    let w1c;
    let w00 = get_00_wire(&wi, &wj, gate_id);
    if !wi.w1.bit(0) && !wj.w1.bit(0) {
        w0c = &w00 ^ delta.clone();
        w1c = w00;
    } else {
        w1c = &w00 ^ delta.clone();
        w0c = w00;
    }
    FreeXORWires { w0: w0c, w1: w1c, delta: delta }
}

pub fn generate_xor_wires(delta: BigUint, wi: &FreeXORWires, wj: &FreeXORWires, _gate_id: &BigUint) -> FreeXORWires {
    let w0c = &wi.w0 ^ &wj.w0;
    let w1c = &w0c ^ delta.clone();
    FreeXORWires { w0: w0c, w1: w1c, delta: delta }
}

pub fn get_00_wire(wi: &FreeXORWires, wj: &FreeXORWires, gate_id: &BigUint) -> BigUint {
    for left in [&wi.w0, &wi.w1] {
        for right in [&wj.w0, &wj.w1] {
            if !left.bit(0) && !right.bit(0) {
                return gc_kdf_128(&left, &right, gate_id)
            }
        }
    }
    panic!("Couldn't find where both wires lsb was 0");
}