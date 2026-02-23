use num_bigint::{BigUint};

use crate::gates::gates::GateType;
pub trait Evaluator {
    fn evaluate_gate(wi: &BigUint, wj: &BigUint, gate_type : &GateType, gate_id: &BigUint, table: &Vec<BigUint>) -> BigUint {
        match gate_type {
            GateType::AND => Self::evaluate_and_gate(wi, wj, gate_id, table),
            GateType::XOR => Self::evaluate_xor_gate(wi, wj, gate_id, table),
        }
    }

    fn evaluate_and_gate(wi: &BigUint, wj: &BigUint, gate_id: &BigUint, table: &Vec<BigUint>) -> BigUint;
    fn evaluate_xor_gate(wi: &BigUint, wj: &BigUint, gate_id: &BigUint, table: &Vec<BigUint>) -> BigUint;
}