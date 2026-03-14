use std::{collections::HashMap, ops::Add, vec};

use crate::gates::gate_gen::{GateType};
use num_bigint::{BigUint, ToBigUint};
// Responsible for creating "recipes" for the gates. Garbler will construct a circuit based on this recipe, creating the wires and output tables.

// Each gate has a build id, where the output wire of the gate has the same id. 
// This way we can provide two wire id's from other gates as input, and ensure to provide the correct values. The wire id does not neccesarlly correlate to the id of the gate genereated in wire_gen.

pub struct CircuitBuilder {
    gates : Vec<GateBuild>,
    outputs_created: BigUint,
    true_constant : WireBuild, 
    false_constant : WireBuild,
    branches : HashMap<BigUint, Vec<GateBuild>> // output wire id of branch, branch
}

#[derive(Debug)]
pub struct CircuitBuild {
    gates : Vec<GateBuild>,
    true_constant : WireBuild, 
    false_constant : WireBuild,
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
        let branches = HashMap::new();
        let true_constant = WireBuild::new(0.to_biguint().unwrap(), 0.to_biguint().unwrap());
        let false_constant = WireBuild::new(0.to_biguint().unwrap(), 1.to_biguint().unwrap());

        CircuitBuilder {
            gates: gates, outputs_created: 2.to_biguint().unwrap(), true_constant: true_constant, false_constant: false_constant, branches : branches
        }
    }

    pub fn get_circuit_build(&mut self) -> CircuitBuild {
        self.gates.sort_by_key(|gate| gate.wo().ready_at_layer.clone());
        // self.numerate_gate_branches();
        CircuitBuild { gates : self.gates.clone(), true_constant: self.true_constant.clone(), false_constant: self.false_constant.clone() }
    }

    // fn numerate_gate_branches(&mut self) {
    //     let mut branch_num = 0;
    //     for branch in self.branches.clone() {
    //         for mut gate in branch {
    //             gate.branch = branch_num.to_biguint().unwrap();
    //         }
    //         branch_num += 1;
    //     }
    //     // numerate all branches which can be done now as we know how many we have
    // }

    // An if block where the "code" which needs to be run inside the statement is provided as a part of branch_true. branch_false is conceptually seen as the block which runs after the if, no such thing as if else. If else is an abstraction to a if(true_stamement) { a() } if(!true_stamement) { b() }. To create if else, run two build_if 
    // The block provided are all the prior code (as gates) which has run so far, + the code which needs to run inside the if statement. 
    pub fn build_if(&mut self, boolean : WireBuild, code_block : &mut Vec<GateBuild>) -> WireBuild {
        let current_branch_output = self.gates[self.gates.len() - 1].wo().clone();
        let mut current_branch = self.branches.get(&current_branch_output.wire_id).unwrap_or(&self.gates).clone(); // default, no branch has been added yet
        
        // Insert the new branch 
        code_block.append(&mut current_branch);
        let code_block_output = code_block[code_block.len() - 1].wo();
        self.branches.insert(code_block_output.wire_id.clone(), code_block.clone());

        let neg_boolean = self.build_gate(&boolean, &self.true_constant.clone(), GateType::XOR);
        let and_0 = self.build_gate(code_block_output, &neg_boolean.wo(), GateType::AND);
        let and_1 = self.build_gate(&current_branch_output, &boolean, GateType::AND);
        self.build_gate(&and_0.wo(), &and_1.wo(), GateType::XOR).wo().clone()
    }

    pub fn build_is_equal(&mut self, input_wires: Vec<WireBuild>) ->  WireBuild {
        // Compares each bit in a tree like structure
        let mut garbler_input_wires = Vec::new();
        let mut evaluator_input_wires = Vec::new();
        for i in 0..input_wires.len() {
            if i < (input_wires.len()/2) {
                garbler_input_wires.push(input_wires[i].clone());
            } else {
                evaluator_input_wires.push(input_wires[i].clone());
            }
        }
        let xnor_1 = self.build_xnor(&input_wires[0], &input_wires[2]);
        let xnor_2 = self.build_xnor(&input_wires[1], &input_wires[3]);
        let output = self.build_and(&xnor_1, &xnor_2);
        output
    }

    pub fn build_or(&mut self, input_wi: &WireBuild, input_wj: &WireBuild) -> Vec<GateBuild> {

        let xor_0 = self.build_xor(&input_wi.clone(), &input_wj.clone());
        let and_0 = self.build_and(input_wi, input_wj);
        let xor_1 = self.build_xor(&xor_0, &and_0);
        let output = xor_1.clone();

        vec![xor_0, and_0, xor_1]
    }

    pub fn build_xnor(&mut self, wi: &WireBuild, wj: &WireBuild) -> WireBuild {
        let xor = self.build_gate(wi, wj, GateType::XOR);
        let xor_with_constant = self.build_gate(xor.wo(), &self.true_constant.clone(), GateType::XOR);
        let xnor_output = xor_with_constant.wo().clone();
        
        xnor_output
    }

    pub fn build_and(&mut self, input_wi: &WireBuild, input_wj: &WireBuild) -> WireBuild {
        let and =self.build_gate(input_wi, input_wj, GateType::AND);
        and.wo().clone()
    }

    pub fn build_xor(&mut self, input_wi: &WireBuild, input_wj: &WireBuild) -> WireBuild {
        let xor = self.build_gate(input_wi, input_wj, GateType::XOR);
        xor.wo().clone()
    }

    pub fn build_input_wires(&mut self, amount : u32) -> Vec<WireBuild> {
        let mut input_wires = vec![];
        for _i in 0..amount {
            let input_wire = WireBuild::new(0.to_biguint().unwrap(), self.outputs_created.clone());
            input_wires.push(input_wire);
            self.outputs_created += 1.to_biguint().unwrap();
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
    branch: BigUint
}

impl GateBuild {
    pub fn new(gate_type: GateType, wi: WireBuild, wj: WireBuild, wo: WireBuild) -> Self {
        Self::new_with_branch(gate_type, wi, wj, wo, 0.to_biguint().unwrap())
    }
    pub fn new_with_branch(gate_type: GateType, wi: WireBuild, wj: WireBuild, wo: WireBuild, branch : BigUint) -> Self {
        GateBuild {
            gate_type,
            wi,
            wj,
            wo,
            branch
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
