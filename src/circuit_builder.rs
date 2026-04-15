use core::fmt;
use std::{cmp::max, collections::{HashMap, HashSet, VecDeque}};

use crate::gates::gate_gen::GateType;
use num_bigint::{BigUint, ToBigUint};
// Responsible for creating "recipes" for the gates. Garbler will construct a circuit based on this recipe, creating the wires and output tables.

// Each gate has a build id, where the output wire of the gate has the same id.
// This way we can provide two wire id's from other gates as input, and ensure to provide the correct values. The wire id does not neccesarilly correlate to the id of the gate generated in wire_gen.

type StackID = BigUint;
type GateID = BigUint;
pub struct CircuitBuilder {
    gates: HashMap<GateID, GateBuild>,
    stacks: HashMap<StackID, StackBuild>,
    false_constant: WireBuild,
    true_constant: WireBuild,
    wires_created : BigUint, // Maybe remove this
    garbler_wires: Vec<WireBuild>,
    evaluator_wires : Vec<WireBuild>,
    output_wires: Vec<WireBuild>,
}
#[derive(Clone, Debug)]
pub enum Build {
    Gate(GateBuild),
    Stack(StackBuild)
}

#[derive(Clone, Debug)]
pub struct StackBuild {
    pub input_wire: WireBuild,
    pub output_wire: WireBuild,
    pub conditional: WireBuild,
    pub true_circuit: Vec<Build>,
    pub false_circuit: Vec<Build>,
    pub id : StackID
}

#[derive(Debug, Clone)]
pub struct CircuitBuild {
    pub gates: Vec<GateBuild>,
    pub output_wires: Vec<WireBuild>,
    pub garbler_wires: Vec<WireBuild>,
    pub evaluator_wires: Vec<WireBuild>
}

impl CircuitBuild {
    pub fn get_gates(&self) -> &Vec<GateBuild> {
        &self.gates
    }
}

impl CircuitBuilder {
    pub fn new() -> Self {
        let gates = HashMap::new();
        let stacks = HashMap::new();
        let false_constant = WireBuild::new(0, 0.to_biguint().unwrap());
        let true_constant = WireBuild::new(0, 1.to_biguint().unwrap());
        let garbler_wires = Vec::new();
        let evaluator_wires = Vec::new();
        let output_wires = Vec::new();

        CircuitBuilder {
            gates,
            stacks,
            false_constant,
            wires_created : 2.to_biguint().unwrap(),
            true_constant,
            garbler_wires,
            evaluator_wires,
            output_wires
        }
    }

    // Tell which wires should be used for garbler and evaluator input
    // Should perhaps make a method which creates a new build with a certain input and deletes the old build.
    // For now you just need to call set input wires
    pub fn set_input_wires(&mut self, input_length : u64) -> (Vec<WireBuild>, Vec<WireBuild>) {
        let garbler_wires = self.build_input_wires(input_length);
        self.garbler_wires = garbler_wires.clone();
        let evaluator_wires = self.build_input_wires(input_length);
        self.evaluator_wires = evaluator_wires.clone();
        (garbler_wires, evaluator_wires)
    }

    pub fn get_circuit_build(&mut self) -> CircuitBuild {
        if self.garbler_wires.len() == 0 || self.evaluator_wires.len() == 0 {
            panic!("Input wires not set")
        }
        let mut gates_as_vec: Vec<GateBuild> = self.gates.values().cloned().collect();
        if self.gates.len() > 0 {
            // self.numerate_gate_branches();
            gates_as_vec.sort_by_key(|gate| gate.wo().ready_at_layer.clone());
        }
        CircuitBuild {
            gates: gates_as_vec,
            output_wires: self.output_wires.clone(),
            garbler_wires : self.garbler_wires.clone(),
            evaluator_wires : self.evaluator_wires.clone(),
        }
    }

    pub fn print_circuit(&mut self) {
        let cb = self.get_circuit_build();
        println!("{:}", " ***** CIRCUIT_BUILD ***** ");
        println!("Amount of gates: {}", cb.gates.len());
        for gate in &cb.gates {
            println!("{}", gate);
        }
    }

    pub fn build_stacked_if(&mut self, cond : &WireBuild, true_output : &Vec<WireBuild>, false_output : &Vec<WireBuild>) -> Vec<WireBuild> { // False output should be the input wires to the true output
        let mut output = vec![];
        let (padded_true, padded_false) = self.pad_input(true_output, false_output);
        for output_index in 0..padded_true.len() { // Go through all output wires and add a stacked impl, note |padded_true| == |padded_false|
            let true_wire = &padded_true[output_index];
            let false_wire = &padded_false[output_index];
            let branch_id = self.stacks.len() * 2;

            // Find subcircuit for c0 and c1. Traverse from each of their output wire until reaching intital input wire
            let c0 = self.get_subcircuit(false_wire.wire_id());
            // self.annotate_with_branch(&c0, &branch_id);
            let c1 = self.get_subcircuit(true_wire.wire_id());
            // self.annotate_with_branch(&c1, &(branch_id + 1));

            // Use the first wire in the true circuit as input wire 
            // c1.sort_by_key(|gate| gate..ready_at_layer.clone());
            let input_wire = false_wire; // The false wire is the situation where we dont change the wire  

            // Create output wire
            let compute_layer = true_wire.ready_at_layer.clone().max(false_wire.ready_at_layer.clone()) + 1;
            let output_wire = WireBuild::new(compute_layer, self.wires_created.clone());
            self.increment_wires_created();

            let stack_build = StackBuild { input_wire : input_wire.clone(), output_wire : output_wire.clone(), conditional : cond.clone(), true_circuit: c1, false_circuit: c0, id: branch_id.to_biguint().unwrap()};
            self.stacks.insert(branch_id.to_biguint().unwrap(), stack_build);
            
            output.push(output_wire);
        }
        output
    }

    // An if block where a block of gates, derived from the output of them, is added depending on a boolean. MUX always has an else.
    // This is more like an if else. Just make an if. 
    pub fn build_if(
        &mut self,
        conditional: &WireBuild,
        true_output: &Vec<WireBuild>,
        false_output: &Vec<WireBuild>
    ) -> Vec<WireBuild> {
        let true_constant = &self.true_constant.clone();
        let mut output = vec![];
        let (padded_true, padded_false) = self.pad_input(true_output, false_output);

        for i in 0..padded_false.len() {
            let true_bit = &padded_true[i];
            let false_bit = &padded_false[i];
            let neg_boolean = self.build_gate(&conditional, &true_constant, GateType::XOR);
            let and_0 = self.build_gate(true_bit, &neg_boolean, GateType::AND);
            let and_1 = self.build_gate(false_bit, conditional, GateType::AND);
            let output_wire = self.build_gate(&and_0, &and_1, GateType::XOR);
            output.push(output_wire);
        }
        self.set_output_wires(output.clone());
        output
    }

    /*
    Routine implementing multiplication.
     */
    pub fn build_multiplier(&mut self, input_wires_a: Vec<WireBuild>, input_wires_b: Vec<WireBuild>) -> Vec<WireBuild> {
        let mut partial_sums: VecDeque<Vec<WireBuild>> = VecDeque::new();
        for (index_b, bit_b) in input_wires_b.iter().enumerate() {
            let mut partial_sum: Vec<WireBuild> = Vec::new();

            for _i in 0..index_b {
                partial_sum.push(self.false_constant.clone().clone());
            }

            for bit_a in &input_wires_a {
                let and = self.build_gate(bit_a, bit_b, GateType::AND);
                partial_sum.push(and);
            }

            for _j in index_b..input_wires_b.len() {
                partial_sum.push(self.false_constant.clone().clone());
            }
            partial_sums.push_back(partial_sum);
        }
        while partial_sums.len() > 1 {
            let partial_sum_a = partial_sums.pop_front().unwrap();
            let partial_sum_b = partial_sums.pop_front().unwrap();
            partial_sums.push_back(self.adder(&partial_sum_a, &partial_sum_b, false)); // addition should not produce a 1-carry bit
        }
        let result: Vec<WireBuild> = partial_sums.pop_front().unwrap();
        self.output_wires = result.clone();
        result
    }

    /*
    Routine implementing addition.
     */
    pub fn build_adder(&mut self, input_wires_a: &Vec<WireBuild>, input_wires_b: &Vec<WireBuild>) -> Vec<WireBuild> {
        let result_wires = self.adder(&input_wires_a, input_wires_b, true);
        self.set_output_wires(result_wires.clone());
        result_wires
    }


    pub fn build_is_equal(&mut self, input_wires_a: &Vec<WireBuild>, input_wires_b: &Vec<WireBuild>) -> WireBuild {
        // Compares each bit in a tree like structure
        let (padded_a, padded_b) = self.pad_input(input_wires_a, input_wires_b);
        let mut deque: VecDeque<WireBuild> = VecDeque::new();
        for i in 0..padded_a.len() {
            deque.push_back(self.build_gate(&padded_a[i], &padded_b[i], GateType::XNOR));
        }
        while deque.len() > 1 {
            let first = deque.pop_front().unwrap();
            let second = deque.pop_front().unwrap();
            deque.push_back(self.build_gate(&first, &second, GateType::AND));
        }
        let output = deque.pop_front().unwrap();
        self.set_output_wires(vec![output.clone()]);
        output
    }

    pub fn build_and(&mut self, wi: &WireBuild, wj: &WireBuild) -> Vec<WireBuild> {
        vec![self.build_gate(wi, wj, GateType::AND)]
    }

    pub fn build_input_wires(&mut self, amount: u64) -> Vec<WireBuild> {
        let mut input_wires = vec![];
        for _i in 0..amount {
            let input_wire = WireBuild::new(0, self.wires_created.clone());
            input_wires.push(input_wire);
            self.wires_created += 1.to_biguint().unwrap();
        }
        input_wires
    }

    fn adder(&mut self, input_wires_a: &Vec<WireBuild>, input_wires_b: &Vec<WireBuild>, with_carry : bool) -> Vec<WireBuild> {
        let mut result_wires: Vec<WireBuild> = Vec::new();
        let (padded_a, padded_b) = self.pad_input(input_wires_a, input_wires_b);

        // Build 1 HALF ADDER for first bits of each input
        let mut sum = self.build_gate(&padded_a[0], &padded_b[0], GateType::XOR);
        result_wires.push(sum.clone());
        let mut carry = self.build_gate(&padded_a[0], &padded_b[0], GateType::AND);
        if input_wires_a.len() == 1 {
            result_wires.push(carry.clone());
        }


        // Build FULL ADDERS for all bits but the first
        for i in 1..padded_a.len() {
            let a_wire = &padded_a[i];
            let b_wire = &padded_b[i];
            // SUM - is added to the result wire
            let a_xor_b = self.build_gate(a_wire, b_wire, GateType::XOR);
            sum = self.build_gate(&a_xor_b, &carry.clone(), GateType::XOR);
            result_wires.push(sum);
            // CARRY - is not added to the result wire
            let first_and = self.build_gate(&a_xor_b, &carry, GateType::AND);
            let second_and = self.build_gate(a_wire, b_wire, GateType::AND);
            carry = self.build_gate(&first_and, &second_and, GateType::OR); 
        }
        // The last carry bit needs to be appended to the result (though not in the case where we add 1-bit numbers or if we use the adder for multiplication)
        if input_wires_a.len() != 1 && with_carry {
            result_wires.push(carry);
        }
        result_wires
    }

    // Ensures length of input_a and input_b is equal by adding padding
    fn pad_input(&self, input_a : &Vec<WireBuild>, input_b : &Vec<WireBuild>) -> (Vec<WireBuild>, Vec<WireBuild>) {
        let required_bits = max(input_a.len(), input_b.len());
        let false_constant = &self.false_constant.clone();
        let mut padded_input_a = vec![];
        let mut padded_input_b = vec![];
        for i in 0..required_bits {
            let a_bit = input_a.get(i).unwrap_or(false_constant); // unwrap or set to 0 if the input needs padding, is this stupid?
            padded_input_a.push(a_bit.clone());
            let b_bit = input_b.get(i).unwrap_or(false_constant);
            padded_input_b.push(b_bit.clone());

        }
        (padded_input_a, padded_input_b)
    }

    fn set_output_wires(&mut self, output_wires: Vec<WireBuild>) {
        self.output_wires = output_wires;
    }

    // Gets dependent gates and stacks of start_gate_id until there is no more inputs
    fn get_subcircuit(&mut self, start_gate_id: &BigUint) -> Vec<Build> {
        let mut stack = vec![start_gate_id.clone()];
        let mut subcircuit : Vec<Build> = vec![];

        while let Some(gate_id) = stack.pop() {
            // if we hit a stack remove it as a global circuit and put inside as part of the subcircuit.
            if self.stacks.contains_key(&gate_id) {
                if let Some(stack) = self.stacks.remove(&gate_id) {
                    subcircuit.push(Build::Stack(stack));
                    break;
                }
            }
            // Should not contain same gate twice
            // if subcircuit.contains(&gate_id) {
            //     continue;
            // }
            if let Some(gate) = self.gates.get(&gate_id) { // Might need to include initial wires also, which is not a gate but a wire. Might also need to be more clear about why we use a vec of wirebuilds instead of a vec of gatebuilds. 
                subcircuit.push(Build::Gate(gate.clone()));
                let left_wire = gate.wi().wire_id().clone();
                let right_wire = gate.wj().wire_id().clone();
                stack.push(left_wire);
                stack.push(right_wire);
            }
        }
        subcircuit
    }

    // Annotates all provided gates with the branch id
    fn annotate_with_branch(&mut self, gates : &Vec<GateBuild>, branch_id : &usize) {
        for gate_to_annotate in gates {
            if let Some(gate) = self.gates.get_mut(gate_to_annotate.wo().wire_id()) {
                gate.branches.insert(branch_id.clone());
            }
        }
        
    }

    // Builds a gate with a new id and returns the output wire which also contains when the gate should be calculated
    fn build_gate(&mut self, wi: &WireBuild, wj: &WireBuild, gate_type: GateType) -> WireBuild {
        let compute_layer = wi.ready_at_layer.clone().max(wj.ready_at_layer.clone()) + 1;
        let wo = WireBuild::new(compute_layer, self.wires_created.clone());
        self.increment_wires_created();

        let gate: GateBuild = GateBuild::new(gate_type, wi.clone(), wj.clone(), wo.clone());
        self.gates.insert(wo.wire_id().clone(), gate.clone());
        gate.wo
    }

    fn increment_wires_created(&mut self) {
        self.wires_created += 1u32;
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct WireBuild {
    ready_at_layer: i32,
    wire_id: BigUint,
}

impl WireBuild {
    pub fn new(ready_at_layer: i32, wire_id: BigUint) -> Self {
        WireBuild {
            ready_at_layer,
            wire_id,
        }
    }
    pub fn ready_at_layer(&self) -> &i32 {
        &self.ready_at_layer
    }
    pub fn wire_id(&self) -> &BigUint {
        &self.wire_id
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct GateBuild {
    pub gate_type: GateType,
    wi: WireBuild,
    wj: WireBuild,
    wo: WireBuild,
    branches: HashSet<usize>,
}

impl GateBuild {
    pub fn new(gate_type: GateType, wi: WireBuild, wj: WireBuild, wo: WireBuild) -> Self {
        let branches = HashSet::new();
        GateBuild {
            gate_type,
            wi,
            wj,
            wo,
            branches,
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

impl fmt::Display for GateBuild {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:?}] : id: {:<3} | ready at layer {:?} | Branches: {:?}",
            self.gate_type, self.wo.wire_id, self.wo.ready_at_layer, self.branches
        )
    }
}
