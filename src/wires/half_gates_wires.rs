use num_bigint::BigUint;
use crate::gates::gates::GateType;
use crate::wires::wires::{Wire, Wires};
use crate::crypto_utils::{gc_kdf_128, generate_label_lsb, generate_label, gc_kdf_hg};

pub struct HalfGateWires {
    pub delta: BigUint
}

impl HalfGateWires {
    pub fn delta(&self) -> &BigUint {
        &self.delta
    }
}

impl Wires for HalfGateWires {
    fn new() -> Self {
        let delta = generate_label_lsb(true); // to ensure point and permute holds
        HalfGateWires { delta}
    }

    fn generate_input_wire(&self) -> Wire {
        let delta = self.delta();
        let w0 = generate_label();
        let w1 = &w0 ^ delta;
        Wire::new(w0, w1)
    }
    fn generate_output_wire(&self, wi: &Wire, wj: &Wire, gate: &GateType, gate_id: &BigUint) -> Wire {
        let delta = self.delta();
        match gate {
            GateType::AND=>generate_and_wires(delta, &wi, &wj, gate_id),
            GateType::XOR=>generate_xor_wires(delta, &wi, &wj, gate_id),
        }
    }
}

pub fn generate_and_wires(delta: &BigUint, wi: &Wire, wj: &Wire, index: &BigUint) -> Wire {
    let w0c;
    let w1c;
    let pa = wi.w0().bit(0);
    let pb = wj.w0().bit(0);
    let j0 = index;
    let j1 = j0 + 1u32;
    let tg;
    if pb {
        tg = gc_kdf_hg(&wi.w0(), j0) ^ gc_kdf_hg(&wi.w1(), j0) ^ delta;
    } else {
        tg = gc_kdf_hg(&wi.w0(), j0) ^ gc_kdf_hg(&wi.w1(), j0);
    }
    let wg;
    if pa {
        wg = gc_kdf_hg(&wi.w0(), j0) ^ tg;
    } else {
        wg = gc_kdf_hg(&wi.w0(), j0)
    }

    let te = gc_kdf_hg(&wj.w0(), &j1) ^ gc_kdf_hg(&wj.w1(), &j1) ^ wi.w0();
    let we;
    if pb {
        we = gc_kdf_hg(&wj.w0(), &j1) ^ te ^ wi.w0();
    } else {
        we = gc_kdf_hg(&wj.w0(), &j1)
    }
    w0c = wg ^ we;
    w1c = &w0c ^ delta;

    Wire::new(w0c, w1c)
}
pub fn generate_xor_wires(delta: &BigUint, wi: &Wire, wj: &Wire, _gate_id: &BigUint) -> Wire {
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