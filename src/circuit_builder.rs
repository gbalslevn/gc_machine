use std::ops::Add;
use std::collections::VecDeque;

use crate::{gates::gate_gen::GateType};
use num_bigint::{BigUint, ToBigUint};

// Responsible for creating "recipes" for the gates. Garbler will construct a circuit based on this recipe, creating the wires and output tables. 

// Each gate has a build id, where the output wire of the gate has the same id. 
// This way we can provide two wire id's from other gates as input, and ensure to provide the correct values. The wire id does not neccesarlly correlate to the id of the gate genereated in wire_gen.

pub struct CircuitBuilder {
    gates : Vec<GateBuild>,
    outputs_created: BigUint,
    true_constant : WireBuild, 
    false_constant : WireBuild
}

pub struct CircuitBuild {
    gates : Vec<GateBuild>,
    true_constant : WireBuild, 
    false_constant : WireBuild
}

impl CircuitBuild {
    pub fn get_gates(&self) -> &Vec<GateBuild> {
        &self.gates
    }
    pub fn get_true_constant(&self) -> &WireBuild {
        &self.true_constant
    }
    pub fn get_false_constant(&self) -> &WireBuild {
        &self.false_constant
    }
}

impl CircuitBuilder {
    pub fn new() -> Self {
        let gates = vec![];
        let true_constant = WireBuild::new(0.to_biguint().unwrap(), 0.to_biguint().unwrap());
        let false_constant = WireBuild::new(0.to_biguint().unwrap(), 1.to_biguint().unwrap());

        CircuitBuilder {
            gates: gates, outputs_created: 2.to_biguint().unwrap(), true_constant: true_constant, false_constant: false_constant
        }
    }

    pub fn get_circuit_build(&mut self) -> CircuitBuild {
        self.gates.sort_by_key(|gate| gate.wo().ready_at_layer.clone());
        CircuitBuild { gates : self.gates.clone(), true_constant: self.true_constant.clone(), false_constant: self.false_constant.clone() }
    }

    pub fn build_is_equal(&mut self, input_length : u64) ->  WireBuild {
        // Compares each bit in a tree like structure
        let mut deq = VecDeque::new(); 
        let input_wires = self.build_input_wires(input_length as u32 * 2);
        // Compare initial layer
        for wire in input_wires.chunks(2) {
            let wi = &wire[0];
            let wj = &wire[1];
            let xnor_output = self.build_xnor(wi, wj);
            
            deq.push_back(xnor_output);
        }
        // Binary tree reduction
        while deq.len() > 1 { // perhaps cleaner if we could calculate how many gates is needed, avoiding while loop.
            let element_0 = deq.pop_front().unwrap().clone();
            let element_1 = deq.pop_front().unwrap().clone();
            let xnor_output = self.build_xnor(&element_0, &element_1);
            deq.push_back(xnor_output); 
        }

        let output  = self.gates[self.gates.len() - 1].wo().clone(); 
        output
    }

    pub fn build_or(&mut self, input_wi: &WireBuild, input_wj: &WireBuild) -> WireBuild { // or gate needs 4 input wires
        let input_wi_copy = WireBuild::new(input_wi.wire_id.clone(), input_wi.ready_at_layer.clone() + 1u32.to_biguint().unwrap());
        let input_wj_copy = WireBuild::new(input_wj.wire_id.clone(), input_wj.ready_at_layer.clone() + 1u32.to_biguint().unwrap());

        let xor_0 = self.build_gate(input_wi, input_wj, GateType::XOR);
        let and_0 = self.build_gate(&input_wi_copy, &input_wj_copy, GateType::AND);
        let xor_1 = self.build_gate(&xor_0.wo(), &and_0.wo(), GateType::XOR);
        let output = xor_1.wo().clone();

        output
    }


    // Builds all gates needed to create a xnor, returns them and the final output 
    pub fn build_xnor(&mut self, wi: &WireBuild, wj: &WireBuild) -> WireBuild {
        let xor = self.build_gate(wi, wj, GateType::XOR);
        let xor_with_constant = self.build_gate(xor.wo(), &self.true_constant.clone(), GateType::XOR);
        let xnor_output = xor_with_constant.wo().clone();

        xnor_output
    }

    pub fn build_input_wires(&mut self, amount : u32) -> Vec<WireBuild> {
        let mut input_wires = vec![];
        for _i in 0..amount {
            let input_wire = WireBuild::new(0.to_biguint().unwrap(), self.outputs_created.clone());
            input_wires.push(input_wire);
        }
        input_wires
    }

    // Builds a gate with a new id and the output wire containing when the gate should be calculated
    fn build_gate(&mut self, wi: &WireBuild, wj: &WireBuild, gate_type: GateType) -> GateBuild {
        let compute_layer = wi.ready_at_layer.clone().max(wj.ready_at_layer.clone());
        let one = 1.to_biguint().unwrap();
        let wo = WireBuild::new(compute_layer.add(one), self.outputs_created.clone());
        self.increment_outputs_created();
        
        let gate = GateBuild::new(gate_type, wi.clone(), wj.clone(), wo);
        self.gates.push(gate.clone());
        gate
    }
    fn increment_outputs_created(&mut self) {
        self.outputs_created += 1u32;
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct WireBuild { 
    ready_at_layer: BigUint,
    wire_id: BigUint,
}

impl WireBuild {
    pub fn new(ready_at_layer: BigUint, wire_id : BigUint) -> Self {
        WireBuild {
            ready_at_layer,
            wire_id,
        }
    }
    pub fn output_layer(&self) -> &BigUint {
        &self.ready_at_layer
    }
    pub fn wire_id(&self) -> &BigUint {
        &self.wire_id
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct GateBuild {
    gate_type: GateType,
    wi: WireBuild,
    wj: WireBuild,
    wo: WireBuild,
}

impl GateBuild {
    pub fn new(gate_type: GateType, wi: WireBuild, wj: WireBuild, wo: WireBuild) -> Self {
        GateBuild {
            gate_type,
            wi,
            wj,
            wo,
        }
    }
    pub fn gate_type(&self) -> &GateType {
        &self.gate_type
    }
    pub fn wi(&self) -> &WireBuild {
        &self.wi
    }
    pub fn wj(&self) -> &WireBuild {
        &self.wj
    }
    pub fn wo(&self) -> &WireBuild {
        &self.wo
    }
}
