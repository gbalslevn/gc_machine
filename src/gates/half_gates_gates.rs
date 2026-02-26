use num_bigint::BigUint;
use crate::gates::gates::{Gate, GateType, Gates};
use crate::wires::wires::{Wire, Wires};

pub struct HalfGatesGates<W: Wires> {
    pub wires: W,
}

impl<W: Wires> Gates<W> for HalfGatesGates<W> {
    fn new(wires: W) -> Self {
        HalfGatesGates { wires }
    }

    fn generate_gate(&self, gate: GateType, wi: Wire, wj: Wire, gate_id: BigUint) -> Gate {
        let wo = self.wires.generate_output_wire(&wi, &wj, &gate, &gate_id);
        let tt = self.get_tt(&wi, &wj, &wo, &gate);
        match gate {
            GateType::AND=> {
                let table = generate_and_table(&tt, &gate_id);
                Gate { gate_id, gate_type: GateType::AND, table, wi: wi, wj: wj, wo: wo }
            }
            GateType::XOR=> {
                Gate { gate_id, gate_type: GateType::XOR, table: Vec::new(), wi: wi, wj: wj, wo: wo }
            }
        }
    }

    fn generate_and_table(tt: &[(BigUint, BigUint, BigUint); 4], gate_id: &BigUint) -> Vec<BigUint> {
        let mut table = vec![BigUint::from(0u8); 2];
    }
}