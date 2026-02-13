use num_bigint::BigUint;
use rand::{thread_rng, Rng};
use crate::wires::wires::Wires;
use crate::crypto_utils::{gc_kdf_128, generate_label_lsb};

pub struct GRR3Wires;

impl Wires for GRR3Wires {
    fn generate_input_wires() -> (BigUint, BigUint) {
        let mut rng = thread_rng();
        let choice = rng.gen_bool(1.0 / 2.0);
        let w0 = generate_label_lsb(choice);
        let w1 = generate_label_lsb(!choice);
        (w0, w1)
    }
    fn generate_output_wires(w0i: &BigUint, w1i: &BigUint, w0j: &BigUint, w1j: &BigUint, gate: String, gate_id: &BigUint) -> (BigUint, BigUint) {
        match gate.as_str() {
            "and"=>generate_and_wires(w0i, w1i, w0j, w1j, gate_id),
            "xor"=>generate_xor_wires(w0i, w1i, w0j, w1j, gate_id),
            _=>panic!("Unknown gate {}", gate),
        }
    }
}

pub fn generate_and_wires(w0i: &BigUint, w1i: &BigUint, w0j: &BigUint, w1j: &BigUint, gate_id: &BigUint) -> (BigUint, BigUint) {
    let w0c;
    let w1c;
    let w00 = get_00_wire((w0i, w1i), (w0j, w1j), gate_id);
    if w1i.bit(0) && w1j.bit(0) {
        w0c = generate_label_lsb(!w00.bit(0));
        w1c = w00;
    } else {
        w1c = generate_label_lsb(!w00.bit(0));
        w0c = w00;
    }
    (w0c, w1c)
}

pub fn generate_xor_wires(w0i: &BigUint, w1i: &BigUint, w0j: &BigUint, w1j: &BigUint, gate_id: &BigUint) -> (BigUint, BigUint) {
    let w0c;
    let w1c;
    let w00 = get_00_wire((w0i, w1i), (w0j, w1j), gate_id);
    if (w0i.bit(0) && w1j.bit(0)) || (w1i.bit(0) && w0j.bit(0)) {
        w0c = generate_label_lsb(!w00.bit(0));
        w1c = w00;
    } else {
        w1c = generate_label_lsb(!w00.bit(0));
        w0c = w00;
    }
    (w0c, w1c)
}

pub fn get_00_wire(wi: (&BigUint, &BigUint), wj: (&BigUint, &BigUint), gate_id: &BigUint) -> BigUint {
    for left in [wi.0, wi.1] {
        for right in [wj.0, wj.1] {
            if !left.bit(0) && !right.bit(0) {
                return gc_kdf_128(left, right, gate_id)
            }
        }
    }
    panic!("Couldn't find where both wires lsb was 0");
}