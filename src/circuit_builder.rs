use core::fmt;
use std::{cmp::max, collections::{HashMap, HashSet, VecDeque}};

use crate::gates::gate_gen::GateType;
use num_bigint::{BigUint, ToBigUint};
// Responsible for creating "recipes" for the gates. Garbler will construct a circuit based on this recipe, creating the wires and output tables.

// Each gate has a build id, where the output wire of the gate has the same id.
// This way we can provide two wire id's from other gates as input, and ensure to provide the correct values. The wire id does not neccesarilly correlate to the id of the gate generated in wire_gen.

pub struct CircuitBuilder {
    gates: Vec<GateBuild>,
    outputs_created: BigUint,
    false_constant: WireBuild,
    true_constant: WireBuild,
    garbler_wires : Vec<WireBuild>,
    evaluator_wires : Vec<WireBuild>,
    branches: HashMap<BigUint, Vec<BranchEntry>>, // Markers for a branch with the mux output gate as key. Each key holds all mux for each input bit
    branch_counter: usize,
    output_wires: Vec<WireBuild>,
}

#[derive(Clone, Debug)]
pub struct BranchEntry {
    pub true_id: BigUint,
    pub false_id: BigUint,
    pub boolean_id : BigUint,
    pub mux_gates: Vec<BigUint>,
}

#[derive(Debug, Clone)]
pub struct CircuitBuild {
    pub gates: Vec<GateBuild>,
    pub output_wires: Vec<WireBuild>,
    pub garbler_wires : Vec<WireBuild>,
    pub evaluator_wires : Vec<WireBuild>
}

impl CircuitBuild {
    pub fn get_gates(&self) -> &Vec<GateBuild> {
        &self.gates
    }
}

impl CircuitBuilder {
    pub fn new() -> Self {
        let gates = Vec::new();
        let branches = HashMap::new(); // Branches for each bit 
        let false_constant = WireBuild::new(0, 0.to_biguint().unwrap());
        let true_constant = WireBuild::new(0, 1.to_biguint().unwrap());
        let garbler_wires = Vec::new();
        let evaluator_wires = Vec::new();
        
        let branch_counter = 0;
        let output_wires = Vec::new();

        CircuitBuilder {
            gates: gates,
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
    // Should perhaps make a method which creates a new build with a certain input.
    // For now you just need to call set input wires
    pub fn set_input_wires(&mut self, input_length : u32) -> (Vec<WireBuild>, Vec<WireBuild>) {
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

    pub fn print_circuit(&self) {
        println!("{:}", " ***** CIRCUIT_BUILD ***** ");
        println!("Branches: {}", self.branches.len() + 1);
        println!("Amount of gates: {}", self.gates.len());
        for gate in &self.gates {
            println!("{}", gate);
        }
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
        let mut branches = vec![];
        for i in 0..padded_false.len() {
            let true_bit = &padded_true[i];
            let false_bit = &padded_false[i];
            let neg_boolean = self.build_gate(&boolean, &true_constant, GateType::XOR);
            let and_0 = self.build_gate(true_bit, &neg_boolean, GateType::AND);
            let and_1 = self.build_gate(false_bit, boolean, GateType::AND);
            let output_bit = self.build_gate(&and_0, &and_1, GateType::XOR);
            let branch = BranchEntry {
                true_id: true_bit.wire_id().clone(),
                false_id: false_bit.wire_id().clone(),
                boolean_id : boolean.wire_id().clone(),
                mux_gates: vec![
                    neg_boolean.wire_id().clone(),
                    and_0.wire_id().clone(),
                    and_1.wire_id().clone(),
                    output_bit.wire_id().clone(),
                ],
            };
            branches.push(branch);
            output.push(output_bit);
        }
        // Insert each branch as a key, representing a bit, and set the value as the bits which the bit belongs to.
        for branch in &branches {
            let mux_id = &branch.mux_gates[3];
            self.branches.insert(mux_id.clone(), branches.clone()); // This creates a lot of dublication in the map. Maybe there is some better way of checking if a gate is a branch output, and if its for the correct bit. 
        } 
        self.set_output_wires(output.clone());
        output
    }

    pub fn build_adder(&mut self, input_wires_a: &Vec<WireBuild>, input_wires_b: &Vec<WireBuild>) -> Vec<WireBuild> {
        let result_wires = self.adder(input_wires_a, input_wires_b);
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

    pub fn build_input_wires(&mut self, amount: u32) -> Vec<WireBuild> {
        let mut input_wires = vec![];
        for _i in 0..amount {
            let input_wire = WireBuild::new(0, self.outputs_created.clone());
            input_wires.push(input_wire);
            self.outputs_created += 1.to_biguint().unwrap();
        }
        input_wires
    }

    fn adder(&mut self, input_wires_a: &Vec<WireBuild>, input_wires_b: &Vec<WireBuild>) -> Vec<WireBuild> {
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
        // The last carry bit needs to be appended to the result (though not in the case where we add 1-bit numbers)
        if input_wires_a.len() != 1 {
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

    fn next_branch_id(&mut self) -> usize {
        self.branch_counter += 1;
        self.branch_counter
    }

    // Traverses backwards from the output gates to recursevely propagate all correct branches to each gate
    fn numerate_gate_branches(&mut self) {
        let gate_index_map: HashMap<BigUint, usize> = self.gates.iter().enumerate().map(|(idx, gate)| (gate.wo().wire_id().clone(), idx)).collect();
        let output_wires = self.output_wires.clone();
        // Iterate for each output bit position
        for (bit_position, wire) in output_wires.iter().enumerate() {
            self.branch_counter = 0;
            self.proceed_with_branch(0, &mut vec![], wire.wire_id(), &gate_index_map, bit_position, &mut HashSet::new()); // initial call to recursive loop
        }
    }

    // Adds branch for dependent gates of start_gate_id and splits in a recursive call if reaching a branch
    fn proceed_with_branch(&mut self, branch_id: usize, branch_route: &mut Vec<BigUint>, start_gate_id: &BigUint, gate_lookup : &HashMap<BigUint, usize>, bit_position : usize, visited_this_branch: &mut HashSet<(BigUint, usize)>) {
        // let mut visited_gates: HashSet<BigUint> = HashSet::new(); // Only add branch id once
        let mut stack = vec![start_gate_id.clone()];
        let branches = self.branches.clone();

        while let Some(gate_id) = stack.pop() {
            if !visited_this_branch.insert((gate_id.clone(), branch_id)) {
                continue;
            }
            // if we hit id of a mux, only bit 0 will repropogate start branch out and numerate both branches, else it will just break
            if let Some(branch) = branches.get(&gate_id) {
                // bit position 0 is responsible for calling the methods so the next branches are traversed
                if bit_position == 0 {
                    let next_branch_id = self.next_branch_id();
                    for (i, entry) in branch.iter().enumerate() { // A bit string of length n, has n mux. We iterate all of those
                        // Add both branch_ids to all of the gates in the MUX
                        for mux_gate_id in &entry.mux_gates {
                            if let Some(&gate_idx) = gate_lookup.get(mux_gate_id) { 
                                let gate = &mut self.gates[gate_idx];
                                gate.add_branch(branch_id);
                                gate.add_branch(next_branch_id);
                            }
                        }

                        // true direction proceeds with next_branch_id and adds all prior gates with that next_branch_id
                        let true_id = &entry.true_id;
                        branch_route.push(true_id.clone());
                        self.proceed_with_branch(next_branch_id, branch_route, &true_id, gate_lookup, i, visited_this_branch);
                        self.add_branch_until_brancing(&gate_id, next_branch_id, branch_route, gate_lookup, i, visited_this_branch);

                        // boolean direction proceeds with both next_branch_id and branch_id
                        let boolean_id = &entry.boolean_id;
                        branch_route.pop();
                        branch_route.push(boolean_id.clone());
                        self.proceed_with_branch(branch_id, branch_route, &boolean_id, gate_lookup, i, visited_this_branch);
                        self.proceed_with_branch(next_branch_id, branch_route, &boolean_id, gate_lookup, i, visited_this_branch);

                        // False direction simply proceeds with same branch
                        let false_id = &entry.false_id;
                        branch_route.pop();
                        branch_route.push(false_id.clone());
                        self.proceed_with_branch(branch_id, branch_route, &false_id, gate_lookup, i, visited_this_branch);


                    }
                }
            break;
            }
            // keep traversing backwards
            if let Some(&gate_idx) = gate_lookup.get(&gate_id) {
                let gate = &mut self.gates[gate_idx];
                gate.add_branch(branch_id);
                let left_wire = gate.wi().wire_id().clone();
                let right_wire = gate.wj().wire_id().clone();
                stack.push(left_wire);
                stack.push(right_wire);
            }
        }
    }

    // Adds branch for gates leading to the gate where the branching happened
    fn add_branch_until_brancing(
        &mut self,
        branch_gate_id: &BigUint,
        branch_id: usize,
        branch_route: &mut Vec<BigUint>,
        gate_lookup : &HashMap<BigUint, usize>,
        bit_position : usize,
        visited_this_branch: &mut HashSet<(BigUint, usize)> 
    ) {
        let final_gate_id = self.gates[self.gates.len() - 1].wo().wire_id();
        let mut stack = vec![final_gate_id.clone()];
        let branches = self.branches.clone();

        let mut route_index = 0;
        while let Some(gate_id) = stack.pop() {
            let gate_has_been_visited = !visited_this_branch.insert((gate_id.clone(), branch_id));
            if gate_has_been_visited {
                continue;
            }
            if let Some(&gate_idx) = gate_lookup.get(&gate_id) {
                let gate = &mut self.gates[gate_idx];
                if gate.wo().wire_id() == branch_gate_id {
                    break;
                }
                gate.add_branch(branch_id);

                if let Some(branch) = branches.get(&gate_id) {
                    let entry = &branch[bit_position];
                    // Determine if the gate_id is a branch wire, and then only push the wire which the branch should take
                    stack.push(branch_route[route_index].clone());
                    route_index += 1;
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
