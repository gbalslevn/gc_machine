use core::fmt;
use std::{cmp::max, collections::{HashMap, HashSet, VecDeque}};

use crate::gates::gate_gen::GateType;
use num_bigint::{BigUint, ToBigUint};
// Responsible for creating "recipes" for the gates. Garbler will construct a circuit based on this recipe, creating the wires and output tables.

// Each gate has a build id, where the output wire of the gate has the same id.
// This way we can provide two wire id's from other gates as input, and ensure to provide the correct values. The wire id does not neccesarilly correlate to the id of the gate generated in wire_gen.

type StackID = usize;
type GateID = BigUint;
pub struct CircuitBuilder {
    gates: HashMap<GateID, GateBuild>,
    stacks: HashMap<StackID, StackBuild>,
    false_constant: WireBuild,
    true_constant: WireBuild,
    wires_created : BigUint, 
    garbler_wires: Vec<WireBuild>,
    evaluator_wires : Vec<WireBuild>,
    output_wires: Vec<WireBuild>,
    builds_buffer: Vec<Build>
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SubcircuitBuild {
    pub builds: Vec<Build>,
    pub output_wires: Vec<WireBuild>,
    pub input_wires: Vec<WireBuild>,
}

#[derive(Clone)]
pub struct CircuitBuild {
    pub builds: Vec<Build>,
    pub output_wires: Vec<WireBuild>,
    pub garbler_wires: Vec<WireBuild>,
    pub evaluator_wires: Vec<WireBuild>
}

impl CircuitBuild {
    pub fn get_builds(&self) -> &Vec<Build> {
        &self.builds
    }
}

impl fmt::Display for CircuitBuild {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "--- Circuit Build ---")?;
        writeln!(f, "Builds: {}) ---", self.builds.len())?;
        for (i, build) in self.builds.iter().enumerate() {
            writeln!(f, "Build {:>3}: {}", i, build)?;
        }
        writeln!(f, "------------------------")?;
        Ok(())
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
        let builds_buffer = Vec::new();

        CircuitBuilder {
            gates,
            stacks,
            false_constant,
            wires_created : 2.to_biguint().unwrap(),
            true_constant,
            garbler_wires,
            evaluator_wires,
            output_wires,
            builds_buffer
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
        let gates: Vec<GateBuild> = self.gates.values().cloned().collect();
        let stacks: Vec<StackBuild> = self.stacks.values().cloned().collect();
        
        // Map to Build type and combine
        let mut builds: Vec<Build> = gates.into_iter().map(Build::Gate).collect();
        let mut stack_builds: Vec<Build> = stacks.into_iter().map(Build::Stack).collect(); 
        builds.append(&mut stack_builds);
        if builds.len() > 0 {
            builds.sort_by_key(|build| build.ready_at_layer().clone());
        }
        CircuitBuild {
            builds: builds,
            output_wires: self.output_wires.clone(),
            garbler_wires : self.garbler_wires.clone(),
            evaluator_wires : self.evaluator_wires.clone(),
        }
    }

    pub fn build_stacked_if(&mut self, cond : &WireBuild, if_circuit : &mut Vec<Build>, if_circuit_output : &mut Vec<WireBuild>, else_circuit : &mut Vec<Build>, else_circuit_output : &mut Vec<WireBuild>) -> Vec<WireBuild> { 
        // input wires are derived implicitely from the input wires of if and else circuit. We combine them to find all input wires needed for both subcircuits 
        let if_circuit_inputs = get_input_wires(if_circuit.clone());
        let else_circuit_inputs = get_input_wires(else_circuit.clone());
        let combined_input: HashSet<WireBuild> = if_circuit_inputs.into_iter().chain(else_circuit_inputs.into_iter()).collect();
        let input_wires: Vec<WireBuild> = combined_input.into_iter().collect();
        let false_constant = &self.false_constant.clone();
        
        // Add padding to if neccesary to ensure equal output length of subcircuits 
        if if_circuit_output.len() > else_circuit_output.len() { 
            for _ in 0..if_circuit_output.len() - else_circuit_output.len() {
                let zero_padding = self.build_gate(false_constant, false_constant, GateType::AND).clone();
                else_circuit.push(Build::Gate(zero_padding.clone()));
                else_circuit_output.push(zero_padding.wo);
            }
        }
        if if_circuit_output.len() < else_circuit_output.len() { 
            for _ in 0..else_circuit_output.len() - if_circuit_output.len() {
                let zero_padding = self.build_gate(false_constant, false_constant, GateType::AND);
                if_circuit.push(Build::Gate(zero_padding.clone()));
                if_circuit_output.push(zero_padding.wo);
            }
        }

        if_circuit.sort_by_key(|build| *build.ready_at_layer());
        else_circuit.sort_by_key(|build| *build.ready_at_layer());
    
        let if_output_layer = if_circuit[if_circuit.len() - 1].ready_at_layer(); // Questionable whether this works, different output wires might be ready at different output layers. So simply taking the last wire is not robust.
        let else_output_layer = else_circuit[else_circuit.len() - 1].ready_at_layer();
        let compute_layer = if_output_layer.clone().max(else_output_layer.clone()) + 1;
        
        let c0 = SubcircuitBuild {builds: if_circuit.clone(), output_wires: if_circuit_output.clone(), input_wires: input_wires.clone()};
        let c1 = SubcircuitBuild {builds: else_circuit.clone(), output_wires: else_circuit_output.clone(), input_wires: input_wires.clone()};

        // Remove builds contained inside of true and false gates from circuitbuilders global parameter as they now belong inside the subcircuit of the stack
        let mut builds_in_stack: HashSet<_> = if_circuit.into_iter().collect();
        let else_set: HashSet<_> = else_circuit.into_iter().collect();
        builds_in_stack.extend(else_set);
        for build in builds_in_stack {
            match build.get_type() {
                BuildType::Gate => {
                    let gate_build = build.unwrap_to_gate();
                    self.gates.remove(gate_build.wo().wire_id()).unwrap();
                }
                BuildType::Stack => {
                    let stack_build = build.unwrap_to_stack();
                    self.stacks.remove(&stack_build.id).unwrap();
                } 
            }
        }

        // Create output wires
        let mut output_wires = vec![];
        for _ in if_circuit_output { // or else circuit output padded length
            let output_wire = WireBuild::new(compute_layer, self.wires_created.clone());
            self.increment_wires_created();
            output_wires.push(output_wire);
        }

        let branch_id = self.stacks.len();
        let stack_build = StackBuild { input_wires : input_wires.clone(), output_wires : output_wires.clone(), conditional : cond.clone(), if_circuit : c0, else_circuit: c1, id: branch_id};
        self.stacks.insert(branch_id, stack_build);
        self.set_output_wires(output_wires.clone());
        
        output_wires
    }

    // An if block where a block of gates, derived from the output of them, is added depending on a boolean. MUX always has an else.
    // This is more like an if else. Just make an if. 
    pub fn build_if(
        &mut self,
        conditional: &WireBuild,
        true_output: &Vec<WireBuild>,
        false_output: &Vec<WireBuild>
    ) -> (Vec<Build>, Vec<WireBuild>) {
        let true_constant = &self.true_constant.clone();
        let mut output = vec![];
        let (padded_true, padded_false) = self.pad_input(true_output, false_output);

        for i in 0..padded_false.len() {
            let true_bit = &padded_true[i];
            let false_bit = &padded_false[i];
            let neg_boolean = self.build_gate(&conditional, &true_constant, GateType::XOR);
            let and_0 = self.build_gate(true_bit, neg_boolean.wo(), GateType::AND);
            let and_1 = self.build_gate(false_bit, conditional, GateType::AND);
            let output_wire = self.build_gate(and_0.wo(), and_1.wo(), GateType::XOR);
            output.push(output_wire.wo);
        }
        self.set_output_wires(output.clone());
        (self.get_latest_builds(), output)
    }

    /*
    Routine implementing multiplication.
     */
    pub fn build_multiplier(&mut self, input_wires_a: &Vec<WireBuild>, input_wires_b: &Vec<WireBuild>) -> BuildBlock {
        // let gates = vec![];
        let mut partial_sums: VecDeque<Vec<WireBuild>> = VecDeque::new();
        for (index_b, bit_b) in input_wires_b.iter().enumerate() {
            let mut partial_sum: Vec<WireBuild> = Vec::new();

            for _i in 0..index_b {
                partial_sum.push(self.false_constant.clone().clone());
            }

            for bit_a in input_wires_a {
                let and = self.build_gate(bit_a, bit_b, GateType::AND);
                partial_sum.push(and.wo().clone());
            }

            for _j in index_b..input_wires_b.len() {
                partial_sum.push(self.false_constant.clone().clone());
            }
            partial_sums.push_back(partial_sum);
        }
        let mut adder_blocks: Vec<BuildBlock> = vec![];
        while partial_sums.len() > 1 {
            let partial_sum_a = partial_sums.pop_front().unwrap();
            let partial_sum_b = partial_sums.pop_front().unwrap();
            let adder_block = self.adder(&partial_sum_a, &partial_sum_b, false);
            adder_blocks.push(adder_block.clone());
            partial_sums.push_back(adder_block.output); // addition should not produce a 1-carry bit
        }
        let result: Vec<WireBuild> = partial_sums.pop_front().unwrap();
        self.output_wires = result.clone();
        let multiplier_builds: Vec<Build> = adder_blocks.into_iter().flat_map(|block| block.builds).collect();
        BuildBlock {output: result, builds : multiplier_builds}
    }

    /*
    Routine implementing addition.
     */
    pub fn build_adder(&mut self, input_wires_a: &Vec<WireBuild>, input_wires_b: &Vec<WireBuild>) -> BuildBlock {
        let block = self.adder(&input_wires_a, input_wires_b, true);
        self.set_output_wires(block.output.clone());
        block
    }


    pub fn build_is_equal(&mut self, input_wires_a: &Vec<WireBuild>, input_wires_b: &Vec<WireBuild>) -> WireBuild {
        // Compares each bit in a tree like structure
        let (padded_a, padded_b) = self.pad_input(input_wires_a, input_wires_b);
        let mut deque: VecDeque<WireBuild> = VecDeque::new();
        for i in 0..padded_a.len() {
            deque.push_back(self.build_gate(&padded_a[i], &padded_b[i], GateType::XNOR).wo);
        }
        while deque.len() > 1 {
            let first = deque.pop_front().unwrap();
            let second = deque.pop_front().unwrap();
            deque.push_back(self.build_gate(&first, &second, GateType::AND).wo);
        }
        let output = deque.pop_front().unwrap();
        self.set_output_wires(vec![output.clone()]);
        self.get_latest_builds();
        output
    }

    pub fn build_and(&mut self, wi: &WireBuild, wj: &WireBuild) -> BuildBlock {
        let gate = self.build_gate(wi, wj, GateType::AND);
        BuildBlock { output: vec![gate.wo], builds: self.get_latest_builds() }
    }

    pub fn build_xor(&mut self, wi: &WireBuild, wj: &WireBuild) -> BuildBlock {
        let gate = self.build_gate(wi, wj, GateType::XOR);
        BuildBlock { output: vec![gate.wo], builds: self.get_latest_builds() }
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

    fn adder(&mut self, input_wires_a: &Vec<WireBuild>, input_wires_b: &Vec<WireBuild>, with_carry : bool) -> BuildBlock {
        let mut result_wires: Vec<WireBuild> = Vec::new();
        let (padded_a, padded_b) = self.pad_input(input_wires_a, input_wires_b);

        // Build 1 HALF ADDER for first bits of each input
        let mut sum = self.build_gate(&padded_a[0], &padded_b[0], GateType::XOR);
        result_wires.push(sum.wo().clone());
        let mut carry = self.build_gate(&padded_a[0], &padded_b[0], GateType::AND);
        if input_wires_a.len() == 1 {
            result_wires.push(carry.wo().clone());
        }


        // Build FULL ADDERS for all bits but the first
        for i in 1..padded_a.len() {
            let a_wire = &padded_a[i];
            let b_wire = &padded_b[i];
            // SUM - is added to the result wire
            let a_xor_b = self.build_gate(a_wire, b_wire, GateType::XOR);
            sum = self.build_gate(a_xor_b.wo(), carry.wo(), GateType::XOR);
            result_wires.push(sum.wo().clone());
            // CARRY - is not added to the result wire
            let first_and = self.build_gate(a_xor_b.wo(), &carry.wo(), GateType::AND);
            let second_and = self.build_gate(a_wire, b_wire, GateType::AND);
            carry = self.build_gate(first_and.wo(), second_and.wo(), GateType::OR); 
        }
        // The last carry bit needs to be appended to the result (though not in the case where we add 1-bit numbers or if we use the adder for multiplication)
        if input_wires_a.len() != 1 && with_carry {
            result_wires.push(carry.wo().clone());
        }
        BuildBlock { output: result_wires, builds: self.get_latest_builds() }
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
        assert_eq!(padded_input_a.len(), padded_input_b.len());
        (padded_input_a, padded_input_b)
    }

    fn set_output_wires(&mut self, output_wires: Vec<WireBuild>) {
        self.output_wires = output_wires;
    }

    fn get_latest_builds(&mut self) -> Vec<Build> {
        let latest_builds = self.builds_buffer.clone();
        if latest_builds.len() == 0 {
            panic!("Could not find any builds in buffer")
        }
        self.builds_buffer = vec![];
        latest_builds
    }

    // Builds a gate with a new id and returns the output wire which also contains when the gate should be calculated
    fn build_gate(&mut self, wi: &WireBuild, wj: &WireBuild, gate_type: GateType) -> GateBuild {
        let compute_layer = wi.ready_at_layer.clone().max(wj.ready_at_layer.clone()) + 1;
        let wo = WireBuild::new(compute_layer, self.wires_created.clone());
        self.increment_wires_created();

        let gate: GateBuild = GateBuild::new(gate_type, wi.clone(), wj.clone(), wo.clone());
        self.gates.insert(wo.wire_id().clone(), gate.clone());
        self.builds_buffer.push(Build::Gate(gate.clone()));
        gate
    }

    fn increment_wires_created(&mut self) {
        self.wires_created += 1u32;
    }
}

 fn get_input_wires(circuit: Vec<Build>) -> Vec<WireBuild> {
    // insert all output wires which is used as input in another gate
    let mut output_wires_used_as_input = HashSet::new();
    for build in &circuit {
        match build.get_type() { // instead of having a switch case, perhaps create a getter for output wires in the build
            BuildType::Gate => {
                let gate = build.unwrap_to_gate();
                output_wires_used_as_input.insert(gate.wo());
            }
            BuildType::Stack => {
                let stack = build.unwrap_to_stack();
                // stack.output_wires
                todo!("Insert output wires for stack")
            }
        }
    }
    // Check if a wire is a input wire 
    let mut input_wires = Vec::new();
    for build in &circuit {
        match build.get_type() {
            BuildType::Gate => {
                let gate = build.unwrap_to_gate();
                let wi_is_output_wire = output_wires_used_as_input.contains(gate.wi());
                let wj_is_output_wire = output_wires_used_as_input.contains(gate.wj());
                
                if !wi_is_output_wire {
                    input_wires.push(gate.wi().clone());
                }
                if !wj_is_output_wire {
                    input_wires.push(gate.wj().clone());
                }
            }
            BuildType::Stack => {
                todo!("Check if input wires is used in another gate")
            }
        }
    }
    
    // let mut input_wires_as_vec : Vec<WireBuild> = input_wires.into_iter().collect();
    // input_wires_as_vec.sort_by_key(|w| w.wire_id.clone());
    // input_wires_as_vec
    input_wires
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
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

#[derive(Clone, Debug)]
pub struct BuildBlock {
    pub output: Vec<WireBuild>,
    pub builds: Vec<Build> 
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Build {
    Gate(GateBuild),
    Stack(StackBuild),
}

#[derive(PartialEq)]
pub enum BuildType {
    Gate,
    Stack
}

impl Build {
    pub fn ready_at_layer(&self) -> &i32 {
        match self {
            Build::Gate(gate) => &gate.wo.ready_at_layer(),
            Build::Stack(stack) => &stack.output_wires[0].ready_at_layer(), // all output wires from the stack has the same ready_at_layer
        }
    }

    pub fn get_type(&self) -> BuildType {
        match self {
            Build::Stack(_) => BuildType::Stack,
            Build::Gate(_) => BuildType::Gate,
        }
    }

    pub fn unwrap_to_stack(&self) -> &StackBuild {
        match self {
            Build::Stack(s) => s,
            Build::Gate(_) => panic!("Called unwrap_to_stack on a Gate"),
        }
    }
    pub fn unwrap_to_gate(&self) -> &GateBuild {
        match self {
            Build::Gate(g) => g,
            Build::Stack(_) => panic!("Called unwrap_to_gate on a Stack"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StackBuild {
    pub input_wires: Vec<WireBuild>,
    pub output_wires: Vec<WireBuild>,
    pub conditional: WireBuild,
    pub if_circuit: SubcircuitBuild,
    pub else_circuit: SubcircuitBuild,
    pub id : StackID
}

#[derive(PartialEq, Clone, Debug, Eq, Hash)]
pub struct GateBuild {
    pub gate_type: GateType,
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

impl fmt::Display for Build {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get_type() {
            BuildType::Gate => {
                let gate_build = self.unwrap_to_gate();
                write!(
                    f,
                    "[{:?}] : id: {:<3} | ready at layer {:?}",
                    gate_build.gate_type, gate_build.wo.wire_id, gate_build.wo.ready_at_layer
                )
            }
            BuildType::Stack => {
                let stack_build = self.unwrap_to_stack();
                write!(
                    f,
                    "Stack id: {:<3} | if_circuit len : {} | else_circuit len : {} ",
                    stack_build.id, stack_build.if_circuit.builds.len(), stack_build.else_circuit.builds.len()
                )
            }
        }
    }
}
