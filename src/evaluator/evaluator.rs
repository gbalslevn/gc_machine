use num_bigint::{BigUint};

use crate::{gates::gates::{Gate, GateType}, wires::wires::Wires};
pub trait Evaluator {
    fn evaluate_gate<W>(wi: &BigUint, wj: &BigUint, gate_id: &BigUint, gate_type: &GateType, gate: &Gate<W>) -> BigUint where W : Wires {
        match gate_type {
            GateType::AND => Self::evaluate_and_gate(wi, wj, gate_id, gate),
            GateType::XOR => Self::evaluate_xor_gate(wi, wj, gate_id, gate),
        }
    }

    fn evaluate_and_gate<W>(key: &BigUint, wj: &BigUint, gate_id: &BigUint, gate: &Gate<W>) -> BigUint;
    fn evaluate_xor_gate<W>(wi: &BigUint, wj: &BigUint, gate_id: &BigUint, gate: &Gate<W>) -> BigUint;
}