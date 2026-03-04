use num_bigint::BigUint;
use rand::{thread_rng, Rng};
use crate::gates::gate_gen::GateType;
use crate::wires::wire_gen::{Wire, WireGen};
use crate::crypto_utils::generate_label_lsb;

#[derive(Clone, Copy)]
pub struct PointAndPermuteWireGen;

impl WireGen for PointAndPermuteWireGen {
    fn new() -> Self {
        Self
    }

    fn generate_input_wire(&self) -> Wire {
        generate_wire()
    }

    fn generate_output_wire(&mut self, _wi: &Wire, _wj: &Wire, _gate: &GateType, _gate_id: &BigUint) -> Wire {
        generate_wire()
    }
}

fn generate_wire() -> Wire {
    let mut rng = thread_rng();
        let choice = rng.gen_bool(1.0 / 2.0);
        let w0 = generate_label_lsb(choice);
        let w1 = generate_label_lsb(!choice);
        Wire::new(w0, w1)
}