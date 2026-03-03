use num_bigint::{BigUint};

use crate::gates::gates::GateType;
pub trait Evaluator {
    fn evaluate_gate(&mut self, wi: &BigUint, wj: &BigUint, gate_type : &GateType, table: &Vec<BigUint>) -> BigUint {
        match gate_type {
            GateType::AND => self.evaluate_and_gate(wi, wj, table),
            GateType::XOR => self.evaluate_xor_gate(wi, wj, table),
        }
    }

    fn evaluate_and_gate(&mut self, wi: &BigUint, wj: &BigUint, table: &Vec<BigUint>) -> BigUint;
    fn evaluate_xor_gate(&mut self, wi: &BigUint, wj: &BigUint, table: &Vec<BigUint>) -> BigUint;
    fn increment_index(&mut self);
    fn get_index(&self) -> &BigUint;
}