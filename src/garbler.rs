use std::cmp::max;
use std::collections::{HashMap, VecDeque};

use crate::circuit_builder::{Build, BuildType, StackBuild, SubcircuitBuild};
use crate::crypto_utils::{gc_kdf, gc_kdf_128};
use crate::evaluator::evaluator::{Evaluator};
use crate::gates::half_gates_gate_gen::HalfGatesGateGen;
use crate::wires::half_gates_wire_gen::HalfGatesWireGen;
use crate::{circuit_builder::CircuitBuild,gates::gate_gen::GateGen,ot::eg_elliptic::{self, CipherText},wires::wire_gen::{Wire, WireGen},
};
use k256::PublicKey;
use num_bigint::{BigUint, ToBigUint};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use crate::evaluator::half_gates_evaluator::HalfGatesEvaluator;

#[derive(Serialize, Deserialize, Debug)]
pub struct Circuit {
    pub material: Vec<Vec<BigUint>>,
    pub constant_wires: Vec<BigUint>,
    pub garbler_input: HashMap<BigUint, BigUint>,
    pub evaluator_input: HashMap<BigUint, (CipherText, CipherText)>,
    pub output_conversion: Vec<[(BigUint, u8); 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    pub demuxes: Vec<Vec<BigUint>>,
    pub m_cond: Vec<Vec<BigUint>>,
    pub muxes: Vec<Vec<BigUint>>,
}

impl Stack {
    // Transforms Stack into material suitable to XOR with material in the Half Gates optimization. 
    fn to_material(&self) -> Vec<Vec<BigUint>> {
        let mut material = vec![];
        // Demux material is split into 4 tables with two entries of 128 bits
        let mut demuxes_material = vec![];
        for demux in &self.demuxes {
            for entry in demux {
                let mut buf = [0u8; 32];
                let bytes = entry.to_bytes_be();
                buf[32 - bytes.len()..].copy_from_slice(&bytes);
                let c0_wire = BigUint::from_bytes_be(&buf[..16]); // first 128 bits
                let c1_wire = BigUint::from_bytes_be(&buf[16..]); // last 128 bits  
                demuxes_material.push(vec![c0_wire, c1_wire]);
            }
        }
        // mux is split into two tables
        let mut muxes_material = vec![];
        for mux in &self.muxes {
            muxes_material.push(vec![mux[0].clone(), mux[1].clone()]);
            muxes_material.push(vec![mux[2].clone(), mux[3].clone()]);
        }
        material.extend(demuxes_material);
        material.extend(self.m_cond.clone());
        material.extend(muxes_material);
        
        material
    }
}

impl Circuit {
    pub fn new(
        material: Vec<Vec<BigUint>>,
        constant_wires: Vec<BigUint>,
        garbler_input: HashMap<BigUint, BigUint>,
        evaluator_input: HashMap<BigUint, (CipherText, CipherText)>,
        output_conversion: Vec<[(BigUint, u8); 2]>,
    ) -> Self {
        Circuit {
            material,
            constant_wires,
            garbler_input,
            evaluator_input,
            output_conversion,
        }
    }
}

pub struct Garbler<G: GateGen> {
    pub gate_gen: G,
}

impl<G: GateGen> Garbler<G> {
    pub fn new(gate_gen: G) -> Self {
        Self { gate_gen: gate_gen }
    }

    pub fn create_circuit(
        &mut self,
        circuit_build: &CircuitBuild,
        garblers_input_choices: &VecDeque<u8>,
        evaluators_input_choices: &VecDeque<[PublicKey; 2]>,
    ) -> Circuit {
        if garblers_input_choices.len() != evaluators_input_choices.len() {
            panic!("Garbler and evaluator input length must be equal")
        }
        let mut material: Vec<Vec<BigUint>> = Vec::new();
        let mut known_wires: HashMap<BigUint, Wire> = HashMap::new();
        let mut output_conversion: Vec<[(BigUint, u8); 2]> = Vec::new();
        let builds = circuit_build.get_builds();
        
        // insert constants for true and false wire into known_wires, to enable eg. NOT gates
        let constant_wires = self.insert_constant_wires(&mut known_wires);
        // insert garbler and evaluators input wires
        let (garbler_inputs, evaluator_inputs) = self.insert_input_wires(
            &mut known_wires,
            &circuit_build,
            &mut garblers_input_choices.clone(),
            &mut evaluators_input_choices.clone(),
        );
        
        self.gate_gen.get_wire_gen().new_rng(); 
        let (material, output_wires) = garble_builds(builds, &mut known_wires, self);
        
        for output_wire in &circuit_build.output_wires {
            let wire = output_wires.get(output_wire.wire_id()).unwrap();
            output_conversion.push([(wire.w0().clone(), 0), (wire.w1().clone(), 1)]);
        }
        Circuit::new(
            material,
            constant_wires,
            garbler_inputs,
            evaluator_inputs,
            output_conversion,
        )
    }

    pub fn generate_stack(&mut self, stack_build : &StackBuild, known_wires: &mut HashMap<BigUint, Wire>) -> (Stack, HashMap<BigUint, Wire>) {
        // insert all input wires
        let mut input_wires = vec![];
        for input_wire in &stack_build.input_wires {
            input_wires.push(known_wires.get(input_wire.wire_id()).unwrap().clone());
        }
        // Generate output wires
        let mut output_wires = Vec::new(); 
        let mut output_wires_map = HashMap::new(); // this is stupid maybe find a better way to return them
        for output_wire in &stack_build.output_wires {
            let wire = self.gate_gen.get_wire_gen().generate_input_wire();
            known_wires.insert(output_wire.wire_id().clone(), wire.clone());
            output_wires_map.insert(output_wire.wire_id().clone(), wire.clone());
            output_wires.push(wire.clone());
        }
        // generate material for all situations
        let mut evaluator = HalfGatesEvaluator::new();
        let seed = known_wires.get(&stack_build.conditional.wire_id()).unwrap().clone();
        let (c0_input_wires, c0, c0_output_wires ) = self.generate_subcircuit(seed.w1(), &stack_build.c0_circuit); // use seed w1 to encrypt c0 such that evaluator will evaluate c0 with w0. From semantics of the stacked garbling paper
        let (c0_garbage_input_wires, __c0_garbage,__c0_garbage_output_wires ) = self.generate_subcircuit(seed.w0(), &stack_build.c0_circuit);
        let (c1_input_wires,c1,c1_output_wires ) = self.generate_subcircuit(seed.w0(), &stack_build.c1_circuit);
        let (c1_garbage_input_wires, __c1_garbage,__c1_garbage_output_wires ) = self.generate_subcircuit(seed.w1(), &stack_build.c1_circuit);
        let m_cond = self.stack_material(&c0, &c1, &seed);
        
        // The demuxes are truth tables that, given a conditional wire and an input wire, outputs the input wire to the branch taken and a fixed garbage input to the branch not taken.
        // To reduce number of entries in demux we choose a specific label from each garbage wire that are used as garbage. We choose w0 but could have used w1 aswell.
        let mut demuxes = vec![];
        let mut c0_garbage_input_labels = vec![];
        let mut c1_garbage_input_labels = vec![];
        for i in 0..input_wires.len() {
            let c0_garbage_input_label = c0_garbage_input_wires[i].w0().clone();
            let c1_garbage_input_label = c1_garbage_input_wires[i].w0().clone();
            c0_garbage_input_labels.push(c0_garbage_input_label.clone());
            c1_garbage_input_labels.push(c1_garbage_input_label.clone());
            let demux = self.generate_demux(&input_wires[i], &seed, &c0_input_wires[i], &c1_input_wires[i], &c0_garbage_input_label, &c1_garbage_input_label);
            demuxes.push(demux);
        }

        // To create the mux, we need to know what the garbage output wire labels the evaluator will have for the branch not taken.
        // To get these, we unstack both subcircuits with the wrong seed. (w1 is wrong for c1, since c1 is encrypted with w0)
        // The unstacked garbage circuits are then evaluated on the fixed garbage input.
        let unstacked_c0_garbage = evaluator.unstack_material(seed.w1(), &m_cond, &stack_build.c1_circuit, &stack_build.c0_circuit);
        let unstacked_c0 = evaluator.unstack_material(seed.w0(), &m_cond, &stack_build.c1_circuit, &stack_build.c0_circuit);
        assert_eq!(unstacked_c0, c0);
        let c0_garbage_output_labels = evaluator.evaluate_subcircuit(c0_garbage_input_labels.clone(), unstacked_c0_garbage, &stack_build.c0_circuit);
        let unstacked_c1_garbage = evaluator.unstack_material(seed.w0(), &m_cond, &stack_build.c0_circuit, &stack_build.c1_circuit);
        let unstacked_c1 = evaluator.unstack_material(seed.w1(), &m_cond, &stack_build.c0_circuit, &stack_build.c1_circuit);
        assert_eq!(unstacked_c1, c1);
        let c1_garbage_output_labels = evaluator.evaluate_subcircuit(c1_garbage_input_labels.clone(), unstacked_c1_garbage, &stack_build.c1_circuit);
        let mut muxes = vec![];
        for i in 0..output_wires.len() {
            let mux = self.generate_mux(&seed, &c0_output_wires[i], &c1_output_wires[i], &c0_garbage_output_labels[i], &c1_garbage_output_labels[i], &output_wires[i]);
            muxes.push(mux);
        }
        (Stack { demuxes, m_cond, muxes }, output_wires_map)
    }

    pub fn generate_subcircuit(&mut self, seed: &BigUint, subcircuit_build: &SubcircuitBuild) -> (Vec<Wire>, Vec<Vec<BigUint>>, Vec<Wire>) {
        let mut gate_gen = HalfGatesGateGen::new_with_seed(seed);  
        let mut known_wires = HashMap::new();
        
        // generate all input wires for the subcircuit with the set seed
        let mut input_wires = vec![];
        for input_wire in &subcircuit_build.input_wires {
            if input_wire.wire_id() == &0.to_biguint().unwrap() {
                let padding_wire = gate_gen.get_wire_gen().generate_input_wire();    
                input_wires.push(padding_wire.clone());
                known_wires.insert(0.to_biguint().unwrap(), padding_wire);
            } else {
                let wire = gate_gen.get_wire_gen().generate_input_wire();
                input_wires.push(wire.clone());
                known_wires.insert(input_wire.wire_id().clone(), wire);
            }
        }  
        let gate_gen = HalfGatesGateGen::new_with_seed(seed);
        let mut subcircuit_garbler = Garbler::new(gate_gen);
        let (material, output_wires) = garble_builds(&subcircuit_build.builds, &mut known_wires, &mut subcircuit_garbler);

        let mut output_wires = vec![];
        for wire_build in &subcircuit_build.output_wires {
            let output_wire = known_wires.get(wire_build.wire_id()).unwrap();
            output_wires.push(output_wire.clone());
        }
        (input_wires, material, output_wires)
    }

    pub fn stack_material(&mut self, c0: &Vec<Vec<BigUint>>, c1: &Vec<Vec<BigUint>>, seed : &Wire) -> Vec<Vec<BigUint>> {
        let mut m_cond = vec![];
        let mut filled_c0 = vec![];
        let mut filled_c1 = vec![];
        let mut c0_seed_material_gen = HalfGatesGateGen::new_with_seed(seed.w1());
        let mut c1_seed_material_gen = HalfGatesGateGen::new_with_seed(seed.w0());
        // insert material for empty tables, the FreeXOR gates
        for table in c0 {
            if table.len() > 0 {
                filled_c0.push(table.clone());
            } else {
                let material = c0_seed_material_gen.get_wire_gen().generate_input_wire();
                let table = vec![material.w0().clone(), material.w1().clone()];
                filled_c0.push(table);
            }
        }
        for table in c1 {
            if table.len() > 0 {
                filled_c1.push(table.clone());
            } else {
                let material = c1_seed_material_gen.get_wire_gen().generate_input_wire();
                let table = vec![material.w0().clone(), material.w1().clone()];
                filled_c1.push(table);
            }
        }
        // Pad so the material has equal length
        let longest_material = max(filled_c0.len(), filled_c1.len());
        let padding = vec![0.to_biguint().unwrap(), 0.to_biguint().unwrap()]; // use seed to generate material instead
        if filled_c0.len() < filled_c1.len() {
            for i in filled_c0.len()..filled_c1.len() {
                filled_c0.push(padding.clone())
            }
        }
        if filled_c0.len() > filled_c1.len() {
            for i in filled_c1.len()..filled_c0.len() {
                filled_c1.push(padding.clone())
            }
        }
        assert_eq!(filled_c0.len(), filled_c1.len());
        for table_index in 0..longest_material {        
            let mut stacked_table = vec![];
            for entry_index in 0..2 {
                let stacked_entry = filled_c0[table_index][entry_index].clone() ^ filled_c1[table_index][entry_index].clone(); // xor both values to stack
                stacked_table.push(stacked_entry);
            } 
            m_cond.push(stacked_table);
        }
        assert_eq!(m_cond.len(), longest_material);
        m_cond
    }

    pub fn create_circuit_input(&self, input: &BigUint, required_bits: u64) -> VecDeque<u8> {
        let mut list = VecDeque::new();
        for i in 0..required_bits {
            let bit = input.bit(i) as u8;
            if bit == 0 {
                list.push_back(0 as u8);
            } else {
                list.push_back(1 as u8);
            }
        }
        list
    }

    fn generate_demux(
        &mut self,
        input_wire: &Wire,
        conditional: &Wire,
        c0_wire: &Wire,
        c1_wire: &Wire,
        garbage_c0_wire: &BigUint,
        garbage_c1_wire: &BigUint,
    ) -> Vec<BigUint> {
        let mut table = vec![BigUint::from(0u8); 4];
        let demux_table = get_demux_tt(input_wire, conditional, c0_wire, c1_wire, garbage_c0_wire, garbage_c1_wire);
        for (cond, input, c0_wire, c1_wire) in demux_table {
            let index = get_position(&input, &cond);
            let entry = gc_kdf(&input, &cond, &self.gate_gen.get_index()) ^ ((c0_wire << 128) | c1_wire);
            table[index] = entry;
        }
        self.gate_gen.increment_index();
        table
    }

    fn generate_mux(
        &mut self,
        conditional: &Wire,
        c0_wire: &Wire,
        c1_wire: &Wire,
        garbage_c0_wire: &BigUint,
        garbage_c1_wire: &BigUint,
        output_wire: &Wire,
    ) -> Vec<BigUint> {
        let mut table = vec![BigUint::from(0u8); 4];
        let mux_table = get_mux_tt(conditional, c0_wire, c1_wire, garbage_c0_wire, garbage_c1_wire, output_wire);
        for (cond, c0_wire, c1_wire, output_wire) in mux_table {
            let index = get_mux_pos(&cond, &c0_wire, &c1_wire);
            let entry = gc_kdf_128(&c0_wire, &c1_wire, &self.gate_gen.get_index()) ^ output_wire;
            table[index] = entry;
        }
        self.gate_gen.increment_index();
        table
    }

    // Inserts the garbler and evaluators input wires
    fn insert_input_wires(
        &mut self,
        known_wires: &mut HashMap<BigUint, Wire>,
        build: &CircuitBuild,
        garblers_input_choices: &mut VecDeque<u8>,
        evaluators_input_choices: &mut VecDeque<[PublicKey; 2]>,
    ) -> (
        HashMap<BigUint, BigUint>,
        HashMap<BigUint, (CipherText, CipherText)>,
    ) {
        let mut rng = self.gate_gen.get_wire_gen().get_rng().clone();
        let garbler_wires = &build.garbler_wires;
        let evaluator_wires = &build.evaluator_wires;

        // Insert garbler input and save the garbler input choice in a map of labels
        let mut garbler_inputs = HashMap::new();
        for wirebuild in garbler_wires {
            let wire = self.gate_gen.get_wire_gen().generate_input_wire();
            let garbler_input_choice = garblers_input_choices.pop_front().unwrap();
            let selected_wirelabel = match garbler_input_choice {
                0 => wire.w0(),
                1 => wire.w1(),
                _ => panic!("Invalid bit value: must be 0 or 1"),
            };
            garbler_inputs.insert(wirebuild.wire_id().clone(), selected_wirelabel.clone());
            known_wires.insert(wirebuild.wire_id().clone(), wire.clone());
        }

        // Insert evaluator input wires and save each label for the wire as a ciphertext where the evaluator can decrypt from OT
        let mut evaluator_inputs = HashMap::new();
        for wirebuild in evaluator_wires {
            let wire = self.gate_gen.get_wire_gen().generate_input_wire();
            let wire_encrypted = self.gen_encrypted_wire(
                &wire,
                &evaluators_input_choices.pop_front().unwrap(),
                &mut rng,
            );
            evaluator_inputs.insert(wirebuild.wire_id().clone(), wire_encrypted.clone());
            known_wires.insert(wirebuild.wire_id().clone(), wire.clone());
        }

        (garbler_inputs, evaluator_inputs)
    }

    fn insert_constant_wires(&mut self, known_wires: &mut HashMap<BigUint, Wire>) -> Vec<BigUint> {
        let mut constant_wires = vec![];
        let true_constant = self.gate_gen.get_wire_gen().generate_input_wire();
        let false_constant = self.gate_gen.get_wire_gen().generate_input_wire();
        known_wires.insert(0.to_biguint().unwrap(), false_constant.clone());
        known_wires.insert(1.to_biguint().unwrap(), true_constant.clone());
        constant_wires.insert(0, false_constant.w0().clone());
        constant_wires.insert(1, true_constant.w1().clone());
        constant_wires
    }

    // Encrypts a wire for the evaluator as a part of OT
    fn gen_encrypted_wire(
        &self,
        wire: &Wire,
        input_choice: &[PublicKey; 2],
        rng: &mut ChaCha20Rng,
    ) -> (CipherText, CipherText) {
        let pk_0 = input_choice[0];
        let wj_0_ct = eg_elliptic::encrypt(rng, &pk_0, wire.w0());
        let pk_1 = input_choice[1];
        let wj_1_ct = eg_elliptic::encrypt(rng, &pk_1, wire.w1());

        let wj_encrypted = (wj_0_ct, wj_1_ct);
        wj_encrypted
    }
}

fn garble_builds<G: GateGen>(builds: &Vec<Build>, known_wires: &mut HashMap<BigUint, Wire>, garbler: &mut Garbler<G>) -> (Vec<Vec<BigUint>>, HashMap<BigUint, Wire>) {
    let mut material = vec![];
    let mut output_wires = HashMap::new();
    for build in builds {
        match build.get_type() {
            BuildType::Gate => {
                let gate = build.unwrap_to_gate();
                let wi = known_wires.get(&gate.wi().wire_id()).unwrap().clone();
                let wj = known_wires.get(&gate.wj().wire_id()).unwrap().clone();
    
                let new_gate = garbler.gate_gen.generate_gate(
                    gate.gate_type().clone(),
                    wi.clone(),
                    wj.clone()
                );
                let output_wire_id = gate.wo().wire_id();
                known_wires.insert(output_wire_id.clone(), new_gate.wo.clone());
                let table = new_gate.to_table();
    
                // Store the ciphertexts for the gate
                material.push(table);
                output_wires.insert(gate.wo().wire_id().clone(), new_gate.wo);
            }
            BuildType::Stack => {
                let stack_build = build.unwrap_to_stack();
                // Use a sub-garbler seeded deterministically from the seed so it matches with the randomness used 
                // let seed = BigUint::from_bytes_be(&garbler.gate_gen.get_wire_gen().get_seed()); 
                // let sub_gate_gen = HalfGatesGateGen::new_with_seed(&seed);
                // let mut sub_garbler = Garbler::new(sub_gate_gen); 
                let (stack, stack_output_wires) = garbler.generate_stack(stack_build, known_wires);
                material.extend(stack.to_material());
                output_wires.extend(stack_output_wires);
            }
        }
    }
    (material, output_wires)
}

fn get_position(wi: &BigUint, wj: &BigUint) -> usize {
    let l = wi.bit(0) as usize;
    let r = wj.bit(0) as usize;
    l * 2 + r
}

fn get_demux_tt(
    input_wire: &Wire,
    conditional: &Wire,
    c0_wire: &Wire,
    c1_wire: &Wire,
    garbage_c0_wire: &BigUint,
    garbage_c1_wire: &BigUint) -> [(BigUint, BigUint, BigUint, BigUint); 4] {
    [(conditional.w1().clone(), input_wire.w0().clone(), garbage_c0_wire.clone(), c1_wire.w0().clone()),
    (conditional.w1().clone(), input_wire.w1().clone(), garbage_c0_wire.clone(), c1_wire.w1().clone()),
    (conditional.w0().clone(), input_wire.w0().clone(), c0_wire.w0().clone(), garbage_c1_wire.clone()),
    (conditional.w0().clone(), input_wire.w1().clone(), c0_wire.w1().clone(), garbage_c1_wire.clone())]
}

fn get_mux_tt(
    conditional: &Wire,
    c0_wire: &Wire,
    c1_wire: &Wire,
    garbage_c0_wire: &BigUint,
    garbage_c1_wire: &BigUint,
    output_wire: &Wire,) -> [(BigUint, BigUint, BigUint, BigUint); 4] {
    [(conditional.w1().clone(), garbage_c0_wire.clone(), c1_wire.w0().clone(), output_wire.w0().clone()),
    (conditional.w1().clone(), garbage_c0_wire.clone(), c1_wire.w1().clone(), output_wire.w1().clone()),
        (conditional.w0().clone(), c0_wire.w0().clone(), garbage_c1_wire.clone(), output_wire.w0().clone()),
        (conditional.w0().clone(), c0_wire.w1().clone(), garbage_c1_wire.clone(), output_wire.w1().clone())]
}

fn get_mux_pos(seed: &BigUint, c0_wire: &BigUint, c1_wire: &BigUint) -> usize {
    let s = seed.bit(0) as usize;
    let i = c0_wire.bit(0) as usize;
    let e = c1_wire.bit(0) as usize;
    s * 2 + (i ^ e)
}