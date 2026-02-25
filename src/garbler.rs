use crate::{circuit_builder::GateBuild, gates::gates::{Gate, GateType, Gates}, wires::wires::Wires};

struct Garbler<G: Gates, W: Wires> {
    gate_type: G,
    wire_type: W,
}

impl<G: Gates, W: Wires> Garbler<G, W> {
    pub fn new(gate: G, wire: W) -> Self {
        Self {
            gate_type: gate,
            wire_type: wire,
        }
    }
    pub fn create_circuit(&self, circuit_build : Vec<GateBuild>) {
        let circuit : Vec<Gate> = vec![];
        for gate in circuit_build {
            if gate.gate_type() == &GateType::AND {
                self.gate_type. new(GateType::AND, gate.wi, wj, id)
            } 
            if gate.gate_type() == &GateType::XOR {
    
            }
        }
    }
}
