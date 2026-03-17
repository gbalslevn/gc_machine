use core::fmt;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::gates::gate_gen::GateType;
use num_bigint::{BigUint, ToBigUint};
// Responsible for creating "recipes" for the gates. Garbler will construct a circuit based on this recipe, creating the wires and output tables.

// Each gate has a build id, where the output wire of the gate has the same id.
// This way we can provide two wire id's from other gates as input, and ensure to provide the correct values. The wire id does not neccesarilly correlate to the id of the gate genereated in wire_gen.

pub struct CircuitBuilder {
    gates: HashMap<BigUint, GateBuild>,
    outputs_created: BigUint,
    true_constant: WireBuild,
    output_layer: BigUint,
    branches: HashMap<BigUint, BranchEntry>, // Markers for a branch with the mux output gate as key
    branch_counter: usize
}

#[derive(Clone)]
pub struct BranchEntry {
    pub true_gate_id: BigUint,
    pub false_gate_id: BigUint,
    pub mux_gates : Vec<GateBuild> 
}

#[derive(Debug, Clone)]
pub struct CircuitBuild {
    pub gates: Vec<GateBuild>,
    pub output_layer: BigUint,
}

impl CircuitBuild {
    pub fn get_gates(&self) -> &Vec<GateBuild> {
        &self.gates
    }
}

impl CircuitBuilder {
    pub fn new() -> Self {
        let gates = HashMap::new();
        let branches = HashMap::new();
        let true_constant = WireBuild::new(0.to_biguint().unwrap(), 1.to_biguint().unwrap());
        let output_layer = 0.to_biguint().unwrap();

        CircuitBuilder {
            gates: gates,
            outputs_created: 2.to_biguint().unwrap(),
            true_constant: true_constant,
            output_layer: output_layer,
            branches,
            branch_counter: 0
        }
    }

    pub fn get_circuit_build(&mut self) -> CircuitBuild {
        let mut gates_list: Vec<_> = self.gates.values().cloned().collect(); 
        gates_list.sort_by_key(|gate| gate.wo().ready_at_layer.clone());
        self.numerate_gate_branches();

        CircuitBuild {
            gates: gates_list,
            output_layer: self.output_layer.clone(),
        }
    }

    pub fn print_summary(&self) {
        println!("{:}", " ***** CIRCUIT_BUILD ***** ");
        println!("Branches: {}", self.branches.len() + 1);
        println!("Amount of gates: {}", self.gates.len());
        for (i, (_gate_id, gate)) in self.gates.iter().enumerate() {
            println!("{:>2} | {}", i, gate); 
        }
    }

    // An if block where a block of gates, derived from the output of them, is added depending on a boolean. MUX always has an else, but in this case we just set the else to do nothing by providing current output.
    pub fn build_if(
        &mut self,
        boolean: &WireBuild,
        true_output: &WireBuild,
        false_output: &WireBuild,
    ) -> WireBuild {
        let neg_boolean = self.build_gate(&boolean, &self.true_constant.clone(), GateType::XOR);
        let and_0 = self.build_gate(true_output, neg_boolean.wo(), GateType::AND);
        let and_1 = self.build_gate(false_output, boolean, GateType::AND);
        let output = self.build_gate(&and_0.wo(), &and_1.wo(), GateType::XOR);
        let mux_id = output.wo().wire_id().clone();
        let branch = BranchEntry { true_gate_id: true_output.wire_id().clone(), false_gate_id : false_output.wire_id().clone(), mux_gates : vec![neg_boolean, and_0, and_1, output.clone()]};
        self.branches.insert(mux_id, branch); 
        output.wo
    }

    pub fn build_is_equal(&mut self, input_wires: Vec<WireBuild>) -> WireBuild {
        // Compares each bit in a tree like structure
        if input_wires.len() % 2 == 1 {
            panic!("Checking for equality requires even number of bits between comparators");
        }
        let mut deque: VecDeque<WireBuild> = VecDeque::new();
        for wires in input_wires.chunks(2) {
            deque.push_back(self.build_xnor(&wires[0], &wires[1]));
        }
        while deque.len() > 1 {
            let first = deque.pop_front().unwrap();
            let second = deque.pop_front().unwrap();
            deque.push_back(self.build_and(&first, &second));
        }
        let output = deque.pop_front().unwrap();
        output
    }

    pub fn build_or(&mut self, input_wi: &WireBuild, input_wj: &WireBuild) -> WireBuild {
        let xor_0 = self.build_gate(input_wi, input_wj, GateType::XOR);
        let and_0 = self.build_gate(input_wi, input_wj, GateType::AND);
        let xor_1 = self.build_gate(xor_0.wo(), and_0.wo(), GateType::XOR);

        xor_1.wo().clone()
    }

    pub fn build_xnor(&mut self, wi: &WireBuild, wj: &WireBuild) -> WireBuild {
        let xor = self.build_gate(wi, wj, GateType::XOR);
        let xor_with_constant =
            self.build_gate(xor.wo(), &self.true_constant.clone(), GateType::XOR);
        let xnor_output = xor_with_constant.wo().clone();

        xnor_output
    }

    pub fn build_and(&mut self, input_wi: &WireBuild, input_wj: &WireBuild) -> WireBuild {
        let and = self.build_gate(input_wi, input_wj, GateType::AND);
        and.wo().clone()
    }

    pub fn build_xor(&mut self, input_wi: &WireBuild, input_wj: &WireBuild) -> WireBuild {
        let xor = self.build_gate(input_wi, input_wj, GateType::XOR);
        xor.wo().clone()
    }

    pub fn build_input_wires(&mut self, amount: u32) -> Vec<WireBuild> {
        let mut input_wires = vec![];
        for _i in 0..amount {
            let input_wire = WireBuild::new(0.to_biguint().unwrap(), self.outputs_created.clone());
            input_wires.push(input_wire);
            self.outputs_created += 1.to_biguint().unwrap();
        }
        input_wires
    }

    fn next_branch_id(&mut self) -> usize {
        self.branch_counter += 1;
        self.branch_counter
    }
    
    // Traverses backwards from the output gate to propogate all correct branches to each gate
    fn numerate_gate_branches(&mut self) {
        let final_gate_id = self.gates[self.gates.len() - 1].wo().wire_id().clone();
        let id_to_gate_index: HashMap<BigUint, usize> = self.gates.iter().enumerate().map(|(idx, gate)| (gate.wo().wire_id().clone(), idx)).collect(); // Lookup to find the index of each gate in gates according to its wire_id. Returns gate in the form of the index in self.gates
        self.proceed_with_branch(0, &mut vec![], &final_gate_id, &id_to_gate_index); // initial call to recursive loop
    }

    // Adds branch for dependent gates of start_gate_id and splits if reaching a branch
    fn proceed_with_branch(&mut self, branch_id: usize, branch_route : &mut Vec<BigUint>, start_gate_id : &BigUint, lookup_index : &HashMap<BigUint, usize>) {
        let mut visited_gates: HashSet<BigUint> = HashSet::new(); // Only add branch id once
        let mut stack = vec![start_gate_id.clone()];

        while let Some(gate_id) = stack.pop() { 
            if !visited_gates.insert(gate_id.clone()) {
                continue;
            }   
            if let Some(&gate_index) = lookup_index.get(&gate_id) {
                if let Some(branch) = self.branches.get(&gate_id).cloned() { // if we hit a branch id, id of the mux (which is the last gate in the mux), start also numerating gates with that new branch
                    let next_branch_id = self.next_branch_id();
                    for gate in branch.mux_gates { // Add both branches to all gates in the MUX
                        if let Some (&mux_gate_idx) = lookup_index.get(gate.wo().wire_id()) {
                            self.gates[mux_gate_idx].add_branch(branch_id);
                            self.gates[mux_gate_idx].add_branch(next_branch_id);
                        }
                    }
                    let true_gate = &branch.true_gate_id;
                    branch_route.push(true_gate.clone());
                    self.proceed_with_branch(next_branch_id, branch_route, true_gate, lookup_index); 
                    self.add_branch_until_brancing(&gate_id, next_branch_id, branch_route, lookup_index);
                    
                    let false_gate = &branch.false_gate_id;
                    branch_route.pop();
                    branch_route.push(false_gate.clone());
                    self.proceed_with_branch(branch_id, branch_route, false_gate, lookup_index); // Simply proceed with the false branch
                    break;
                } else { // keep adding more gates
                    let dependent_gate = &mut self.gates[gate_index];
                    dependent_gate.add_branch(branch_id);
                    let left_wire = dependent_gate.wi().wire_id().clone();
                    let right_wire = dependent_gate.wj().wire_id().clone();
                    stack.push(left_wire);
                    stack.push(right_wire);
                }
            }
        }
    }

    // Adds branch for gates leading to the gate where the branching happens
    fn add_branch_until_brancing(&mut self, branch_gate_id: &BigUint, branch_id: usize, branch_route : &mut Vec<BigUint>, lookup_index : &HashMap<BigUint, usize>) {
        let mut visited_gates: HashSet<BigUint> = HashSet::new(); // Only add branch id once
        let final_gate_id = self.gates[self.gates.len() - 1].wo().wire_id().clone();
        let mut stack = vec![final_gate_id];

        let mut route_index = 0;
        while let Some(gate_id) = stack.pop() {
            if &gate_id == branch_gate_id {
                break;
            }
            if let Some(&gate_index) = lookup_index.get(&gate_id) {
                let consuming_gate = &mut self.gates[gate_index];
                if !visited_gates.insert(consuming_gate.wo().wire_id().clone()) {
                        continue;
                }
                consuming_gate.add_branch(branch_id);
                if let Some(_branch) = self.branches.get(&gate_id) { // Determine if the gate_id is a branch wire, and then only push the wire which the branch should take
                    stack.push(branch_route[route_index].clone());
                    route_index += 1;
                } else {
                    let left_wire = consuming_gate.wi().wire_id().clone();
                    let right_wire = consuming_gate.wj().wire_id().clone();
                    stack.push(left_wire);
                    stack.push(right_wire);
                }
            }
        }
    }

    // Builds a gate with a new id and the output wire containing when the gate should be calculated
    fn build_gate(&mut self, wi: &WireBuild, wj: &WireBuild, gate_type: GateType) -> GateBuild {
        let compute_layer =
            wi.ready_at_layer.clone().max(wj.ready_at_layer.clone()) + 1.to_biguint().unwrap();
        self.output_layer = compute_layer.clone();
        let wo = WireBuild::new(compute_layer, self.outputs_created.clone());
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
    pub fn new(ready_at_layer: BigUint, wire_id: BigUint) -> Self {
        WireBuild {
            ready_at_layer,
            wire_id,
        }
    }
    pub fn ready_at_layer(&self) -> &BigUint {
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
    branches: Vec<usize>,
}

impl GateBuild {
    pub fn new(gate_type: GateType, wi: WireBuild, wj: WireBuild, wo: WireBuild) -> Self {
        let branches = Vec::new();
        GateBuild {
            gate_type,
            wi,
            wj,
            wo,
            branches,
        }
    }

    pub fn add_branch(&mut self, branch_id: usize) {
        self.branches.push(branch_id);
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
    pub fn branches(&self) -> &Vec<usize> {
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
