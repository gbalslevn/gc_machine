use num_bigint::BigUint;
use crate::gates::gate_gen::GateType;
use crate::wires::wire_gen::{Wire, WireGen};
use crate::crypto_utils::{gc_kdf_128, generate_label_lsb, generate_label, gc_kdf_hg};

pub struct HalfGatesWireGen {
    pub delta: BigUint,
    pub tg: BigUint,
    pub te: BigUint,
}

impl HalfGatesWireGen {
    pub fn delta(&self) -> &BigUint {
        &self.delta
    }
    pub fn reset_gate_values(&mut self) {
        self.tg = BigUint::from(0u32);
        self.te = BigUint::from(0u32);
    }

    pub fn tg(&self) -> &BigUint { &self.tg }
    pub fn te(&self) -> &BigUint { &self.te }
}

impl WireGen for HalfGatesWireGen {
    fn new() -> Self {
        let delta = generate_label_lsb(true); // to ensure point and permute holds
        HalfGatesWireGen { delta, tg: BigUint::from(0u32), te: BigUint::from(0u32) }
    }

    fn generate_input_wire(&self) -> Wire {
        let delta = self.delta();
        let w0 = generate_label();
        let w1 = &w0 ^ delta;
        Wire::new(w0, w1)
    }
    fn generate_output_wire(&mut self, wi: &Wire, wj: &Wire, gate: &GateType, gate_id: &BigUint) -> Wire {
        match gate {
            GateType::AND=>generate_and_wires(self, &wi, &wj, gate_id),
            GateType::XOR=>generate_xor_wires(self, &wi, &wj, gate_id),
        }
    }
}

pub fn generate_and_wires(wire_gen: &mut HalfGatesWireGen, wi: &Wire, wj: &Wire, index: &BigUint) -> Wire {
    let pa = wi.w0().bit(0);
    let pb = wj.w0().bit(0);
    let j0 = index;
    let j1 = j0 + 1u32;
    let wi0_hash = gc_kdf_hg(&wi.w0(), j0);
    let wi1_hash = gc_kdf_hg(&wi.w1(), j0);
    let wj0_hash = gc_kdf_hg(&wj.w0(), &j1);
    let wj1_hash = gc_kdf_hg(&wj.w1(), &j1);

    let (tg, wg) = generate_garb_half_gate(pa, pb, wire_gen.delta(), wi0_hash, wi1_hash);
    let (te, we) = generate_eval_half_gate(pb, wi.w0(), wj0_hash, wj1_hash);

    wire_gen.tg = tg;
    wire_gen.te = te;
    let w0c = wg ^ we;
    let w1c = &w0c ^ wire_gen.delta();

    Wire::new(w0c, w1c)
}

pub fn generate_garb_half_gate(pa: bool, pb: bool, delta: &BigUint, wi0_hash: BigUint, wi1_hash: BigUint) -> (BigUint, BigUint) {
    let mut tg = &wi0_hash ^ wi1_hash;
    if pb {
        tg = tg ^ delta;
    }
    let mut wg = wi0_hash;
    if pa {
        wg = wg ^ &tg;
    }
    (tg, wg)
}

pub fn generate_eval_half_gate(pb: bool, wi0: &BigUint, wj0_hash: BigUint, wj1_hash: BigUint) -> (BigUint, BigUint) {
    let te = &wj0_hash ^ wj1_hash ^ wi0;
    let mut we = wj0_hash;
    if pb {
        we = we ^ &te ^ wi0;
    }
    (te, we)
}
pub fn generate_xor_wires(wire_gen: &mut HalfGatesWireGen, wi: &Wire, wj: &Wire, _gate_id: &BigUint) -> Wire {
    let w0c = wi.w0() ^ wj.w0();
    let w1c = &w0c ^ wire_gen.delta();
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