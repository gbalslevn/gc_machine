use core::fmt;
use std::{cmp::max, collections::{HashMap, HashSet, VecDeque}};

use crate::gates::gate_gen::GateType;
use num_bigint::{BigUint, ToBigUint};
// Responsible for creating "recipes" for the gates. Garbler will construct a circuit based on this recipe, creating the wires and output tables.

// Each gate has a build id, where the output wire of the gate has the same id.
// This way we can provide two wire id's from other gates as input, and ensure to provide the correct values. The wire id does not neccesarilly correlate to the id of the gate generated in wire_gen.

type MuxID = BigUint;
type BranchID = usize;
pub struct CircuitBuilder {
    gates: Vec<GateBuild>,
    stacks: Vec<StackBuild>,
    outputs_created: BigUint,
    false_constant: WireBuild,
    true_constant: WireBuild,
    garbler_wires: Vec<WireBuild>,
    evaluator_wires : Vec<WireBuild>,
    branches: HashMap<MuxID, BranchEntry>,
    branch_counter: usize,
    output_wires: Vec<WireBuild>,
}

#[derive(Clone, Debug)]
pub struct BranchEntry {
    pub true_id: BigUint,
    pub false_id: BigUint,
    pub boolean_id: BigUint,
    pub mux_gates: Vec<BigUint>,
    pub branch_id: BranchID,
}

impl BranchEntry {
    // Mux id is the output gate of the mux
    fn get_mux_id(&self) -> &MuxID {
        &self.mux_gates[3]
    }
}

#[derive(Clone, Debug)]
pub struct StackBuild {
    pub demux_build: Vec<DemuxBuild>,
    pub branch_build_left: BranchBuild,
    pub branch_build_right: BranchBuild,
    pub mux_build: Vec<MuxBuild>,
}

#[derive(Clone, Debug)]
pub struct DemuxBuild {
    pub seed: WireBuild,
    pub input_wire: WireBuild,
    pub output_wire_left: WireBuild,
    pub output_wire_right: WireBuild,
}

#[derive(Clone, Debug)]
pub struct MuxBuild {
    pub seed: WireBuild,
    pub input_wire_left: WireBuild,
    pub input_wire_right: WireBuild,
    pub output_wire: WireBuild,
}

#[derive(Clone, Debug)]
pub struct BranchBuild {
    // pub stack_build: Option<StackBuild>,
    // pub gate_build: Option<Vec<GateBuild>>,
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
        let gates = Vec::new();
        let stacks = Vec::new();
        let branches : HashMap<MuxID, BranchEntry> = HashMap::new(); 
        let false_constant = WireBuild::new(0, 0.to_biguint().unwrap());
        let true_constant = WireBuild::new(0, 1.to_biguint().unwrap());
        let garbler_wires = Vec::new();
        let evaluator_wires = Vec::new();
        
        let branch_counter = 0;
        let output_wires = Vec::new();

        CircuitBuilder {
            gates,
            stacks,
            outputs_created: 2.to_biguint().unwrap(),
            false_constant,
            true_constant,
            garbler_wires,
            evaluator_wires,
            branches,
            branch_counter,
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
        if self.gates.len() > 0 {
            self.numerate_gate_branches();
            self.gates.sort_by_key(|gate| gate.wo().ready_at_layer.clone());
        }
        CircuitBuild {
            gates: self.gates.clone(),
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

    pub fn build_conditional(
        &mut self,
        conditional: &WireBuild,
        input_wire: &Vec<WireBuild>
    ) -> WireBuild {

        // let demuxes = Vec::new();
        // for wire in input_wire {
        //     // let demux = DemuxBuild {
        //     //     seed: conditional.clone(),
        //     //     input_wire: wire.clone(),
        //         // output_wire_left: WireBuild {},
        //         // output_wire_right: WireBuild {},
        //     };
        // };
        todo!()
    }

    // An if block where a block of gates, derived from the output of them, is added depending on a boolean. MUX always has an else.
    pub fn build_if(
        &mut self,
        boolean: &WireBuild,
        true_output: &Vec<WireBuild>,
        false_output: &Vec<WireBuild>
    ) -> Vec<WireBuild> {
        let true_constant = &self.true_constant.clone();
        let mut output = vec![];
        let (padded_true, padded_false) = self.pad_input(true_output, false_output);
        let branch_id : BranchID = self.next_branch_id();

        for i in 0..padded_false.len() {
            let true_bit = &padded_true[i];
            let false_bit = &padded_false[i];
            let neg_boolean = self.build_gate(&boolean, &true_constant, GateType::XOR);
            let and_0 = self.build_gate(true_bit, &neg_boolean, GateType::AND);
            let and_1 = self.build_gate(false_bit, boolean, GateType::AND);
            let output_wire = self.build_gate(&and_0, &and_1, GateType::XOR);
            let branch = BranchEntry {
                true_id: true_bit.wire_id().clone(),
                false_id: false_bit.wire_id().clone(),
                boolean_id : boolean.wire_id().clone(),
                mux_gates: vec![
                    neg_boolean.wire_id().clone(),
                    and_0.wire_id().clone(),
                    and_1.wire_id().clone(),
                    output_wire.wire_id().clone(),
                ],
                branch_id : branch_id
            };
            output.push(output_wire);
            self.branches.insert(branch.get_mux_id().clone(), branch.clone());
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
            let input_wire = WireBuild::new(0, self.outputs_created.clone());
            input_wires.push(input_wire);
            self.outputs_created += 1.to_biguint().unwrap();
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

    fn next_branch_id(&mut self) -> BranchID {
        self.branch_counter += 1;
        self.branch_counter
    }

    // Traverses backwards from the output gates to recursevely propagate all correct branches to each gate
    fn numerate_gate_branches(&mut self) {
        let gate_index_map: HashMap<BigUint, usize> = self.gates.iter().enumerate().map(|(idx, gate)| (gate.wo().wire_id().clone(), idx)).collect();
        let output_wires = self.output_wires.clone();
        
        // Numerate recursevely from each output bit
        // A bit string of length n, has n muxes. We iterate all of those
        let max_branch_id : BranchID = self.branch_counter;
        for wire in output_wires {
            self.proceed_with_branch(max_branch_id, &mut HashMap::new(), wire.wire_id(), &gate_index_map, &mut HashSet::new()); 
        }
    }

    // Adds branch for dependent gates of start_gate_id and splits in a recursive call if reaching a branch
    fn proceed_with_branch(&mut self, branch_id: BranchID, branch_route: &mut HashMap<usize, BigUint>, start_gate_id: &BigUint, gate_lookup : &HashMap<BigUint, usize>, visited_branches: &mut HashSet<(BigUint, usize)>) {
        let mut stack = vec![start_gate_id.clone()];
        let branches = self.branches.clone();

        while let Some(gate_id) = stack.pop() {
            let branch_has_been_inserted = !visited_branches.insert((gate_id.clone(), branch_id));
            if branch_has_been_inserted {
                continue;
            }
            // if we hit id of a mux
            if let Some(branch) = branches.get(&gate_id) {
                let next_branch_id = branch.branch_id - 1;
                // Add both branch_ids to all of the gates in the MUX
                for mux_gate_id in &branch.mux_gates {
                    if let Some(&gate_idx) = gate_lookup.get(mux_gate_id) { 
                        let gate = &mut self.gates[gate_idx];
                        gate.add_branch(branch.branch_id);
                        gate.add_branch(next_branch_id);
                    }
                }

                // true direction proceeds with next_branch_id and adds all prior gates with that next_branch_id
                let true_id = &branch.true_id;
                branch_route.insert(branch.branch_id, true_id.clone());
                self.proceed_with_branch(next_branch_id, branch_route, &true_id, gate_lookup, visited_branches);
                self.add_branch_until_brancing(&gate_id, next_branch_id, branch_route, gate_lookup, visited_branches);
                
                // boolean direction proceeds with both next_branch_id and branch_id
                let boolean_id = &branch.boolean_id;
                branch_route.insert(branch.branch_id, boolean_id.clone());
                self.proceed_with_branch(branch_id, branch_route, &boolean_id, gate_lookup, visited_branches);
                self.proceed_with_branch(next_branch_id, branch_route, &boolean_id, gate_lookup, visited_branches);
                
                // False direction simply proceeds with same branch
                let false_id = &branch.false_id;
                branch_route.insert(branch.branch_id, false_id.clone());
                self.proceed_with_branch(branch_id, branch_route, &false_id, gate_lookup, visited_branches);
            
            break;
            }
            // keep traversing backwards
            if let Some(&gate_index) = gate_lookup.get(&gate_id) {
                let gate = &mut self.gates[gate_index];
                gate.add_branch(branch_id);
                let left_wire = gate.wi().wire_id().clone();
                let right_wire = gate.wj().wire_id().clone();
                stack.push(left_wire);
                stack.push(right_wire);
            }
        }
    }

    // Adds branch for gates leading to the mux where the branching happened
    fn add_branch_until_brancing(
        &mut self,
        mux_id: &MuxID,
        branch_id: usize,
        branch_route: &mut HashMap<usize, BigUint>, // (branch_id, direction_gate_id), eg a direction of the false, true or boolean
        gate_lookup : &HashMap<BigUint, usize>,
        visited_branches: &mut HashSet<(BigUint, usize)> // gate_id, branch_id
    ) {
        let final_gate_id = self.gates[self.gates.len() - 1].wo().wire_id();
        let mut stack = vec![final_gate_id.clone()];
        let branches = self.branches.clone();

        while let Some(gate_id) = stack.pop() {
            let branch_has_been_inserted = !visited_branches.insert((gate_id.clone(), branch_id));
            if branch_has_been_inserted {
                continue;
            }
            if let Some(&gate_index) = gate_lookup.get(&gate_id) {
                let gate = &mut self.gates[gate_index];
                if gate.wo().wire_id() == mux_id {
                    break;
                }
                gate.add_branch(branch_id);

                if let Some(branch) = branches.get(&gate_id) {
                    // Determine if the gate_id is a branch wire, and then only push the wire which the branch should take
                    let route_id = branch_route.get(&branch.branch_id).unwrap();
                    stack.push(route_id.clone());
                } else {
                    let left_wire = gate.wi().wire_id().clone();
                    let right_wire = gate.wj().wire_id().clone();
                    stack.push(left_wire);
                    stack.push(right_wire);
                }
            }
        }
    }

    // Builds a gate with a new id and returns the output wire which also contains when the gate should be calculated
    fn build_gate(&mut self, wi: &WireBuild, wj: &WireBuild, gate_type: GateType) -> WireBuild {
        let compute_layer = wi.ready_at_layer.clone().max(wj.ready_at_layer.clone()) + 1;
        let wo = WireBuild::new(compute_layer, self.outputs_created.clone());
        self.increment_outputs_created();

        let gate: GateBuild = GateBuild::new(gate_type, wi.clone(), wj.clone(), wo);
        self.gates.push(gate.clone());
        gate.wo
    }

    fn increment_outputs_created(&mut self) {
        self.outputs_created += 1u32;
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

    pub fn add_branch(&mut self, branch_id: usize) {
        self.branches.insert(branch_id);
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
    pub fn branches(&self) -> &HashSet<usize> {
        &self.branches
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
