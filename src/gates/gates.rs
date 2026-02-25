use num_bigint::{BigUint};

use crate::wires::wires::{Wire, Wires};


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GateType {
    XOR, 
    AND
}

pub trait Gates<W: Wires> {
    fn new(wires: W) -> Self;

    fn generate_gate(&self, gate : GateType, wi: Wire, wj: Wire, gate_id: BigUint) -> Gate;
    fn get_tt(&self, wi: &Wire, wj: &Wire, wo: &Wire, gate: &GateType) -> [(BigUint, BigUint, BigUint); 4] {
        match gate {
            GateType::AND=>self.get_and_tt(wi, wj, wo),
            GateType::XOR=>self.get_xor_tt(wi, wj, wo),
        }
    }
    fn get_xor_tt(&self, wi: &Wire, wj: &Wire, wo: &Wire) -> [(BigUint, BigUint, BigUint); 4] {
        [(wi.w0().clone(), wj.w0().clone(), wo.w0().clone()), (wi.w0().clone(), wj.w1().clone(), wo.w1().clone()), (wi.w1().clone(), wj.w0().clone(), wo.w1().clone()), (wi.w1().clone(), wj.w1().clone(), wo.w0().clone())] // should avoid using clone if wanting performancee
    }
    fn get_and_tt(&self, wi: &Wire, wj: &Wire, wo: &Wire) -> [(BigUint, BigUint, BigUint); 4] {
        [(wi.w0().clone(), wj.w0().clone(), wo.w0().clone()), (wi.w0().clone(), wj.w1().clone(), wo.w0().clone()), (wi.w1().clone(), wj.w0().clone(), wo.w0().clone()), (wi.w1().clone(), wj.w1().clone(), wo.w1().clone())]
    }
}

pub struct Gate {
    pub gate_id: BigUint,
    pub gate_type: GateType,
    pub table : Vec<BigUint>,
    pub wi: Wire,
    pub wj: Wire,
    pub wo: Wire,
}