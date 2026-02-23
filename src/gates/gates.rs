use num_bigint::{BigUint};

use crate::wires::wires::{Wires};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GateType {
    XOR, 
    AND
} 

pub trait Gates<W> where W : Wires {
    fn new(gate_type : GateType, id : BigUint) -> Gate<W>;
    
    fn get_tt(&self, wi: &W, wj: &W, wo: &W, gate: &GateType) -> [(BigUint, BigUint, BigUint); 4] {
        match gate {
            GateType::AND=>self.get_and_tt(wi, wj, wo),
            GateType::XOR=>self.get_xor_tt(wi, wj, wo),
        }
    }
    fn get_xor_tt(&self, wi: &W, wj: &W, wo: &W) -> [(BigUint, BigUint, BigUint); 4] {
        [(wi.w0().clone(), wj.w0().clone(), wo.w0().clone()), (wi.w0().clone(), wj.w1().clone(), wo.w1().clone()), (wi.w1().clone(), wj.w0().clone(), wo.w1().clone()), (wi.w1().clone(), wj.w1().clone(), wo.w0().clone())] // should avoid using clone if wanting performancee
    }
    fn get_and_tt(&self, wi: &W, wj: &W, wo: &W) -> [(BigUint, BigUint, BigUint); 4] {
        [(wi.w0().clone(), wj.w0().clone(), wo.w0().clone()), (wi.w0().clone(), wj.w1().clone(), wo.w0().clone()), (wi.w1().clone(), wj.w0().clone(), wo.w0().clone()), (wi.w1().clone(), wj.w1().clone(), wo.w1().clone())]
    }
}

pub struct Gate<W> {
    pub gate_id: BigUint,
    pub gate_type: GateType,
    pub table : Vec<BigUint>,
    pub wi: W,
    pub wj: W,
    pub wo: W,
}