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

pub trait BuildCount {
    fn get_len(&self) -> usize;
    fn get_material_len(&self) -> usize;
}

impl BuildCount for Vec<Build> {    
    // Returns amount of tables for the list of builds
    fn get_len(&self) -> usize {
        self.iter().map(|b| b.get_gates_len()).sum()
    }
    // Returns amount of tables containing material for the list of builds
    fn get_material_len(&self) -> usize {
        self.iter().map(|b| b.get_material_len()).sum()
    }
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

    // Builds a if which uses stacked garbling 
    pub fn build_stacked_if(&mut self, conditional : &WireBuild, false_block: &mut BuildBlock, true_block: &mut BuildBlock) -> BuildBlock {
        // input wires are derived implicitely from the input wires of c0 and c1. We combine them to find all input wires needed for both subcircuits 
        let c0_inputs = get_input_wires(false_block.builds.clone());
        let c1_inputs = get_input_wires(true_block.builds.clone());
        let padding_wire = WireBuild::new(0, 0.to_biguint().unwrap());
        let combined_input: HashSet<WireBuild> = c0_inputs.clone().into_iter().chain(c1_inputs.clone().into_iter()).collect();
        let mut input_wires: Vec<WireBuild> = combined_input.into_iter().collect();
        input_wires.sort_by_key(|w| w.wire_id.clone());

        // Align inputs in each subcircuit such that they are used with the correct input wires in the mux. 
        let c0_input_set: HashSet<BigUint> = c0_inputs.iter().map(|w| w.wire_id().clone()).collect();
        let c1_input_set: HashSet<BigUint> = c1_inputs.iter().map(|w| w.wire_id().clone()).collect();

        let mut c0_inputs_aligned = vec![];
        let mut c1_inputs_aligned = vec![];
        for combined_wire in &input_wires {
            if c0_input_set.contains(combined_wire.wire_id()) {
                c0_inputs_aligned.push(combined_wire.clone());
            } else {
                c0_inputs_aligned.push(padding_wire.clone());
            }
            if c1_input_set.contains(combined_wire.wire_id()) {
                c1_inputs_aligned.push(combined_wire.clone());
            } else {
                c1_inputs_aligned.push(padding_wire.clone());
            }
        }

        // Find the compute layer of the stack from the input wire with the largest compute layer. When we have all inputs, we can produce output. 
        let mut compute_layer = conditional.ready_at_layer;
        for input_wire in &input_wires {
            if input_wire.ready_at_layer > compute_layer {
                compute_layer = input_wire.ready_at_layer;
            }
        }

        false_block.builds.sort_by_key(|build| *build.ready_at_layer());
        true_block.builds.sort_by_key(|build| *build.ready_at_layer());

        // Create output wires
        let mut output_wires = vec![];
        let output_len = max(false_block.output.len(), true_block.output.len());
        for _ in 0..output_len { 
            let output_wire = WireBuild::new(compute_layer + 1, self.wires_created.clone());
            self.increment_wires_created();
            output_wires.push(output_wire);
        }

        // Add padding if neccesary to ensure equal output length of subcircuits 
        let padding_wire = WireBuild::new(0, 0.to_biguint().unwrap());
        false_block.output.resize(output_len, padding_wire.clone());
        true_block.output.resize(output_len, padding_wire);
        
        let c0 = SubcircuitBuild {builds: false_block.builds.clone(), output_wires: false_block.output.clone(), input_wires: c0_inputs_aligned.clone()};
        let c1 = SubcircuitBuild {builds: true_block.builds.clone(), output_wires: true_block.output.clone(), input_wires: c1_inputs_aligned.clone()};        
       
        // Remove builds contained inside of true and false gates from circuitbuilders global parameters as they now belong inside the subcircuit of the stack
        let mut builds_in_stack: HashSet<_> = false_block.builds.clone().into_iter().collect();
        let else_set: HashSet<_> = true_block.builds.clone().into_iter().collect();
        builds_in_stack.extend(else_set);
        for build in builds_in_stack {
            match build.get_type() {
                BuildType::Gate => {
                    let gate_build = build.unwrap_to_gate();
                    self.gates.remove(gate_build.wo().wire_id());
                }
                BuildType::Stack => {
                    let stack_build = build.unwrap_to_stack();
                    self.stacks.remove(&stack_build.id);
                } 
            }
        }

        let stack_id = self.stacks.len();
        let m_cond_len = max(false_block.builds.get_material_len(), true_block.builds.get_material_len());
        let stack_build = StackBuild { input_wires, output_wires: output_wires.clone(), conditional : conditional.clone(), c0, c1, id: stack_id, m_cond_len};
        self.stacks.insert(stack_id, stack_build.clone());
        self.set_output_wires(output_wires.clone());
        
        BuildBlock { builds: vec![Build::Stack(stack_build)], output: output_wires}
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

// Get all input wires to the build
 fn get_input_wires(circuit: Vec<Build>) -> Vec<WireBuild> {
    // insert all output wires which is used as input in another gate
    let mut output_wires = HashSet::new();
    for build in &circuit {
        match build.get_type() { 
            BuildType::Gate => {
                let gate = build.unwrap_to_gate();
                output_wires.insert(gate.wo());
            }
            BuildType::Stack => {
                let stack = build.unwrap_to_stack();
                output_wires.extend(&stack.output_wires);
            }
        }
    }
    // Check if a wire is a input wire 
    let mut input_wires = Vec::new();
    for build in &circuit {
        match build.get_type() {
            BuildType::Gate => {
                let gate = build.unwrap_to_gate();
                let wi_is_output_wire = output_wires.contains(gate.wi());
                let wj_is_output_wire = output_wires.contains(gate.wj());
                
                if !wi_is_output_wire {
                    input_wires.push(gate.wi().clone());
                }
                if !wj_is_output_wire {
                    input_wires.push(gate.wj().clone());
                }
            }
            BuildType::Stack => {
                let stack = build.unwrap_to_stack();
                input_wires.push(stack.conditional.clone());
                for input_wire in &stack.input_wires {
                    if !(output_wires.contains(input_wire)) {
                        input_wires.push(input_wire.clone());
                    }
                }
            }
        }
    }
    input_wires.sort();
    input_wires.dedup();
    input_wires
}

#[derive(Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
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

#[derive(PartialEq, Debug)]
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

    pub fn get_material_len(&self) -> usize {
        self.get_len(false)
    }

    pub fn get_gates_len(&self) -> usize {
        self.get_len(true)
    }

    fn get_len(&self, with_empty_tables: bool) -> usize {
        match self.get_type() {
            BuildType::Gate => {
                if with_empty_tables {
                    1
                } else {
                    let gate_build = self.unwrap_to_gate();
                    if gate_build.gate_type() != &GateType::XOR && gate_build.gate_type() != &GateType::XNOR {
                        1
                    } else {
                        0
                    }
                }
            },
            BuildType::Stack => {
                let stack_build = self.unwrap_to_stack();
                stack_build.input_wires.len() * 4 + stack_build.m_cond_len + stack_build.output_wires.len() * 2
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StackBuild {
    pub input_wires: Vec<WireBuild>,
    pub output_wires: Vec<WireBuild>,
    pub conditional: WireBuild,
    pub c0: SubcircuitBuild,
    pub c1: SubcircuitBuild,
    pub id : StackID,
    pub m_cond_len: usize
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
                    "Stack id: {:<3} | c0_circuit len : {} | c1_circuit len : {} ",
                    stack_build.id, stack_build.c0.builds.len(), stack_build.c1.builds.len()
                )
            }
        }
    }
}
