use num_bigint::{BigUint};
use strum_macros::{Display, EnumIter};
use crate::{wires::wire_gen::{Wire, WireGen}};


#[derive(Clone, Copy, Debug, PartialEq, Display, EnumIter)]
pub enum GateType {
    XOR,
    XNOR,
    AND,
    NAND,
    OR,
    NOR
}

pub trait GateGen<W: WireGen> {
    fn new(wire_gen: W) -> Self;

    fn generate_gate(&mut self, gate : GateType, wi: Wire, wj: Wire) -> Gate;
    fn get_tt(&self, wi: &Wire, wj: &Wire, wo: &Wire, gate: &GateType) -> [(BigUint, BigUint, BigUint); 4] {
        match gate {
            GateType::XOR=>self.get_xor_tt(wi, wj, wo),
            GateType::XNOR=>self.get_xnor_tt(wi, wj, wo),
            GateType::AND=>self.get_and_tt(wi, wj, wo),
            GateType::NAND=>self.get_nand_tt(wi, wj, wo),
            GateType::OR=>self.get_or_tt(wi, wj, wo),
            GateType::NOR=>self.get_nor_tt(wi, wj, wo),
        }
    }
    fn get_xor_tt(&self, wi: &Wire, wj: &Wire, wo: &Wire) -> [(BigUint, BigUint, BigUint); 4] {
        [(wi.w0().clone(), wj.w0().clone(), wo.w0().clone()), (wi.w0().clone(), wj.w1().clone(), wo.w1().clone()), (wi.w1().clone(), wj.w0().clone(), wo.w1().clone()), (wi.w1().clone(), wj.w1().clone(), wo.w0().clone())] // should avoid using clone if wanting performance
    }
    fn get_xnor_tt(&self, wi: &Wire, wj: &Wire, wo: &Wire) -> [(BigUint, BigUint, BigUint); 4] {
        [(wi.w0().clone(), wj.w0().clone(), wo.w1().clone()), (wi.w0().clone(), wj.w1().clone(), wo.w0().clone()), (wi.w1().clone(), wj.w0().clone(), wo.w0().clone()), (wi.w1().clone(), wj.w1().clone(), wo.w1().clone())]
    }

    fn get_and_tt(&self, wi: &Wire, wj: &Wire, wo: &Wire) -> [(BigUint, BigUint, BigUint); 4] {
        [(wi.w0().clone(), wj.w0().clone(), wo.w0().clone()), (wi.w0().clone(), wj.w1().clone(), wo.w0().clone()), (wi.w1().clone(), wj.w0().clone(), wo.w0().clone()), (wi.w1().clone(), wj.w1().clone(), wo.w1().clone())]
    }
    fn get_nand_tt(&self, wi: &Wire, wj: &Wire, wo: &Wire) -> [(BigUint, BigUint, BigUint); 4] {
        [(wi.w0().clone(), wj.w0().clone(), wo.w1().clone()), (wi.w0().clone(), wj.w1().clone(), wo.w1().clone()), (wi.w1().clone(), wj.w0().clone(), wo.w1().clone()), (wi.w1().clone(), wj.w1().clone(), wo.w0().clone())]
    }
    fn get_or_tt(&self, wi: &Wire, wj: &Wire, wo: &Wire) -> [(BigUint, BigUint, BigUint); 4] {
        [(wi.w0().clone(), wj.w0().clone(), wo.w0().clone()), (wi.w0().clone(), wj.w1().clone(), wo.w1().clone()), (wi.w1().clone(), wj.w0().clone(), wo.w1().clone()), (wi.w1().clone(), wj.w1().clone(), wo.w1().clone())]
    }
    fn get_nor_tt(&self, wi: &Wire, wj: &Wire, wo: &Wire) -> [(BigUint, BigUint, BigUint); 4] {
        [(wi.w0().clone(), wj.w0().clone(), wo.w1().clone()), (wi.w0().clone(), wj.w1().clone(), wo.w0().clone()), (wi.w1().clone(), wj.w0().clone(), wo.w0().clone()), (wi.w1().clone(), wj.w1().clone(), wo.w0().clone())]
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
    pub fn to_table(&self) -> Vec<BigUint> {
        self.table.clone()
    } 
}