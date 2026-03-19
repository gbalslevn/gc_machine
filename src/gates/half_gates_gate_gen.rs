use num_bigint::BigUint;
use crate::gates::gate_gen::{Gate, GateType, GateGen};
use crate::wires::half_gates_wire_gen::HalfGatesWireGen;
use crate::wires::wire_gen::{Wire, WireGen};

pub struct HalfGatesGateGen {
    pub wire_gen: HalfGatesWireGen,
    index: BigUint,

}

impl GateGen<HalfGatesWireGen> for HalfGatesGateGen {
    fn new(wire_gen: HalfGatesWireGen) -> Self {
        HalfGatesGateGen { wire_gen, index: BigUint::from(0u32) }
    }

    fn generate_gate(&mut self, gate: GateType, wi: Wire, wj: Wire) -> Gate {
        let wo = self.wire_gen.generate_output_wire(&wi, &wj, &gate, &self.index);
        match gate {
            GateType::AND => {
                let tg = self.wire_gen.tg().clone();
                let te = self.wire_gen.te().clone();
                self.wire_gen.reset_gate_values();
                let table = vec!(tg, te);
                let gate = Gate { gate_type: GateType::AND, table, wi, wj, wo };
                self.increment_index();
                self.increment_index();
                gate
            }
            GateType::NAND => {
                let tg = self.wire_gen.tg().clone();
                let te = self.wire_gen.te().clone();
                self.wire_gen.reset_gate_values();
                let table = vec!(tg, te);
                let gate = Gate { gate_type: GateType::NAND, table, wi, wj, wo };
                self.increment_index();
                self.increment_index();
                gate
            }
            GateType::XOR => {
                Gate { gate_type: GateType::XOR, table: Vec::new(), wi, wj, wo }
            }
            GateType::XNOR => {
                Gate { gate_type: GateType::XNOR, table: Vec::new(), wi, wj, wo }
            }
            GateType::OR => {
                let tg = self.wire_gen.tg().clone();
                let te = self.wire_gen.te().clone();
                self.wire_gen.reset_gate_values();
                let table = vec!(tg, te);
                let gate = Gate { gate_type: GateType::OR, table, wi, wj, wo };
                self.increment_index();
                self.increment_index();
                gate
            }
            GateType::NOR => {
                let tg = self.wire_gen.tg().clone();
                let te = self.wire_gen.te().clone();
                self.wire_gen.reset_gate_values();
                let table = vec!(tg, te);
                let gate = Gate { gate_type: GateType::NOR, table, wi, wj, wo };
                self.increment_index();
                self.increment_index();
                gate
            }
        }
    }

    fn get_index(&self) -> &BigUint {
        &self.index
    }

    fn increment_index(&mut self) -> &BigUint {
        self.index += 1u32;
        &self.index
    }
}