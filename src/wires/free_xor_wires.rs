use num_bigint::BigUint;
use crate::wires::wires::Wires;
use crate::crypto_utils::{gc_kdf_128, generate_label_lsb, generate_label};

pub struct FreeXORWires {
    delta: BigUint,
}

impl FreeXORWires {
    pub fn new() -> Self {
        Self {
            delta: generate_label_lsb(true), // to ensure point & permute holds
        }
    }
    pub fn delta(&self) -> &BigUint {
        &self.delta
    }
}

impl Wires for FreeXORWires {
    fn generate_input_wires(&self) -> (BigUint, BigUint) {
        let w0 = generate_label();
        let w1 = &w0 ^ &self.delta;
        (w0, w1)
    }
    fn generate_output_wires(&self, wi: &(BigUint, BigUint), wj: &(BigUint, BigUint), gate: String, gate_id: &BigUint) -> (BigUint, BigUint) {
        match gate.as_str() {
            "and"=>generate_and_wires(&self.delta, wi, wj, gate_id),
            "xor"=>generate_xor_wires(&self.delta, wi, wj, gate_id),
            _=>panic!("Unknown gate {}", gate),
        }
    }
}

pub fn generate_and_wires(delta: &BigUint, wi: &(BigUint, BigUint), wj: &(BigUint, BigUint), gate_id: &BigUint) -> (BigUint, BigUint) {
    let w0c;
    let w1c;
    let w00 = get_00_wire(&wi, &wj, gate_id);
    if !wi.1.bit(0) && !wj.1.bit(0) {
        w0c = &w00 ^ delta;
        w1c = w00;
    } else {
        w1c = &w00 ^ delta;
        w0c = w00;
    }
    (w0c, w1c)
}

pub fn generate_xor_wires(delta: &BigUint, wi: &(BigUint, BigUint), wj: &(BigUint, BigUint), _gate_id: &BigUint) -> (BigUint, BigUint) {
    let w0c = &wi.0 ^ &wj.0;
    let w1c = &w0c ^ delta;
    (w0c, w1c)
}

pub fn get_00_wire(wi: &(BigUint, BigUint), wj: &(BigUint, BigUint), gate_id: &BigUint) -> BigUint {
    for left in [&wi.0, &wi.1] {
        for right in [&wj.0, &wj.1] {
            if !left.bit(0) && !right.bit(0) {
                return gc_kdf_128(&left, &right, gate_id)
            }
        }
    }
    panic!("Couldn't find where both wires lsb was 0");
}