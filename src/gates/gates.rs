use num_bigint::{BigUint};

use crate::{garbler::GateEval, wires::wires::{Wire, Wires}};


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GateType {
    XOR, 
    AND
}

pub trait Gates<W: Wires> {
    fn new(wires: W) -> Self;

    fn generate_gate(&mut self, gate : GateType, wi: Wire, wj: Wire) -> Gate;
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
    fn get_index(&self) -> &BigUint;
    fn increment_index(&mut self) -> &BigUint;
}

#[derive(PartialEq)]
pub struct Gate {
    pub gate_type: GateType,
    pub table : Vec<BigUint>,
    pub wi: Wire,
    pub wj: Wire,
    pub wo: Wire,
}

impl Gate {
    pub fn to_gate_eval(&self, gate_id : BigUint, wi_id : BigUint, wj_id : BigUint, is_input_gate : bool) -> GateEval {
        GateEval {gate_id: gate_id, gate_type: self.gate_type, table : self.table.clone(), wi_id: wi_id, wj_id : wj_id, is_input_gate: is_input_gate}
    } 
}