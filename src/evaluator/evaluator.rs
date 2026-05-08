use k256::{PublicKey, SecretKey};
use num_bigint::{BigUint, ToBigUint};
use std::{cmp::max, collections::{HashMap, VecDeque}};

use crate::{
    circuit_builder::{BuildType, StackBuild, SubcircuitBuild}, crypto_utils::{gc_kdf, gc_kdf_128}, evaluator::{half_gates_evaluator::HalfGatesEvaluator}, garbler::{Circuit, Garbler, Stack}, gates::{gate_gen::{GateGen, GateType}, half_gates_gate_gen::HalfGatesGateGen}, ot::eg_elliptic::{self},
};
use crate::circuit_builder::{CircuitBuild};

pub trait Evaluator {
    fn evaluate_gate(
        &mut self,
        wi: &BigUint,
        wj: &BigUint,
        gate_type: &GateType,
        table: &Vec<BigUint>,
    ) -> BigUint {
        match gate_type {
            GateType::AND => self.evaluate_and_gate(wi, wj, table),
            GateType::NAND => self.evaluate_and_gate(wi, wj, table),
            GateType::XOR => self.evaluate_xor_gate(wi, wj, table),
            GateType::XNOR => self.evaluate_xor_gate(wi, wj, table),
            GateType::OR => self.evaluate_and_gate(wi, wj, table),
            GateType::NOR => self.evaluate_and_gate(wi, wj, table),
        }
    }

    fn evaluate_and_gate(
        &mut self,
        wi: &BigUint,
        wj: &BigUint,
        table: &Vec<BigUint>,
    ) -> BigUint;
    fn evaluate_xor_gate(
        &mut self,
        wi: &BigUint,
        wj: &BigUint,
        table: &Vec<BigUint>,
    ) -> BigUint;

    fn evaluate_circuit(
        &mut self,
        circuit_build: &CircuitBuild,
        circuit: Circuit,
        secret_keys: &Vec<(SecretKey, u8)>,
    ) -> u32 {
        let mut known_wires: HashMap<BigUint, BigUint> = HashMap::new(); // id, wire
        let mut stacks: HashMap<BigUint, Stack> = HashMap::new();
        let mut result_wires: Vec<BigUint> = Vec::new();

        if secret_keys.len() != circuit.evaluator_input.len() {
            panic!("Evaluator input length and its secret keys length must be equal")
        }

        // Insert constant values
        known_wires.insert(0.to_biguint().unwrap(), circuit.constant_wires[0].to_biguint().unwrap());
        known_wires.insert(1.to_biguint().unwrap(), circuit.constant_wires[1].to_biguint().unwrap());

        // Insert garblers input wires
        let garbler_hash_keys = circuit.garbler_input.keys().collect::<Vec<_>>();
        for wire_id in garbler_hash_keys {
            let wire = circuit.garbler_input.get(wire_id);
            known_wires.insert(wire_id.clone(), wire.unwrap().clone());
        }
        // Insert evaluator wires
        let mut evaluator_hash_keys = circuit.evaluator_input.keys().collect::<Vec<_>>();
        evaluator_hash_keys.sort();
        let mut secret_keys_iterator = 0;
        for  key in evaluator_hash_keys {
            let evaluator_ciphers = circuit.evaluator_input.get(key).unwrap();
            let evaluator_choice = secret_keys[secret_keys_iterator].1.clone();
            let evaluator_cipher = match evaluator_choice {
                0 => &evaluator_ciphers.0,
                1 => &evaluator_ciphers.1,
                _ => panic!("Invalid evaluator choice"),
            };
            let wire = eg_elliptic::decrypt(&secret_keys[secret_keys_iterator].0, evaluator_cipher);
            known_wires.insert(key.clone(), wire.clone());
            secret_keys_iterator += 1;
        }

        // Evaluate all builds
        let mut index = 0;
        for build in &circuit_build.builds {
            match build.get_type() {
                BuildType::Gate => {
                    let gate = build.unwrap_to_gate();
                    let wi = known_wires.get(&gate.wi().wire_id()).unwrap().clone();
                    let wj = known_wires.get(&gate.wj().wire_id()).unwrap().clone();
                    let result = self.evaluate_gate(&wi, &wj, &gate.gate_type, &circuit.material[index]); // Stacks should be placed at a specific index too. Split into gate like tables. 
                    known_wires.insert(gate.wo().wire_id().clone(), result.clone());
        
                    // Store all result wires
                    if circuit_build.output_wires.contains(gate.wo()) {
                        result_wires.push(result.clone());
                    }
                    index += 1;
                }
                BuildType::Stack => {
                    let stack_build = build.unwrap_to_stack();
                    // We retrive demuxes from received material
                    let mut demuxes = vec![];
                    for _ in &stack_build.input_wires {
                        // 1 demux is 2 tables as it has 4 entries but each entry is 256 bits, so actually its 4 entries, which means we need 4 tables. 
                        let demux_material: Vec<Vec<BigUint>> = circuit.material.get(index..index + 4).expect("Insufficient material for Demux").to_vec(); // get 8 tables from material
                        // Combine the two 128 bit c_0, c_1 input labels for each 4 entries so we can evaluate the demux
                        let entry_0 = &demux_material[0][0] << 128 | &demux_material[0][1];
                        let entry_1 = &demux_material[1][0] << 128 | &demux_material[1][1];
                        let entry_2 = &demux_material[2][0] << 128 | &demux_material[2][1];
                        let entry_3 = &demux_material[3][0] << 128 | &demux_material[3][1];
                        let demux = vec![entry_0, entry_1, entry_2, entry_3];
                        demuxes.push(demux);
                        index += 4;

                    }
                    // We retrive m_cond from material
                    let mut m_cond = vec![];
                    assert_eq!(stack_build.c0_circuit.builds.len(), stack_build.c1_circuit.builds.len());
                    for i in index..index + stack_build.c0_circuit.builds.len() { // Assuming both circuits have equal length
                        let stacked_m_entry = circuit.material[i].clone();
                        m_cond.push(stacked_m_entry);
                    }
                    index += stack_build.c0_circuit.builds.len();
                   
                    // We retrive muxes from material
                    let mut muxes = vec![];
                    for _ in &stack_build.output_wires {
                        let mux_material = circuit.material.get(index..index + 2).expect("Insufficient material for Mux").to_vec(); // 1 mux is 2 tables as it has 4 entries.
                        let mux = vec![mux_material[0][0].clone(), mux_material[0][1].clone(), mux_material[1][0].clone(), mux_material[1][1].clone()];
                        muxes.push(mux);
                        index += 2;
                    }

                    let stack = Stack {demuxes, m_cond, muxes};                                        
                    let stack_output_wires = self.evaluate_stack(stack_build, &stack, &mut known_wires);
                    for i in 0..stack_output_wires.len() {
                        if circuit_build.output_wires.contains(&stack_build.output_wires[i]) {
                            result_wires.push(stack_output_wires[i].clone());
                        }
                    }
                }
            }
        }

        Self::interpret_result(result_wires, &circuit.output_conversion)
    }

    fn evaluate_stack(&mut self, stack_build : &StackBuild, stack : &Stack, known_wires : &mut HashMap<BigUint, BigUint>) -> Vec<BigUint> {
        let mut result_wires = vec![];
        let seed = known_wires.get(stack_build.conditional.wire_id()).unwrap().clone();
        let c0 = self.unstack_material(&seed, &stack.m_cond, &stack_build.c0_circuit);
        let c1 = self.unstack_material(&seed, &stack.m_cond, &stack_build.c1_circuit);
        
        // Get all input wires to the two circuits from demux
        let mut c0_inputs = vec![];
        let mut c1_inputs = vec![];
        for i in 0..stack_build.input_wires.len() {
            let input_wire_id = stack_build.input_wires[i].wire_id();
            let input_wire = known_wires.get(input_wire_id).unwrap().clone();
            let (c0_input, c1_input) = self.evaluate_demux(&input_wire, &seed, &stack.demuxes[i]);
            c0_inputs.push(c0_input);
            c1_inputs.push(c1_input);
        }
        let c0_output = self.evaluate_subcircuit(c0_inputs, c0, &stack_build.c0_circuit);
        let c1_output = self.evaluate_subcircuit(c1_inputs, c1, &stack_build.c1_circuit);
        assert_eq!(c0_output.len(), c1_output.len());

        for i in 0..stack_build.output_wires.len() {
            let output_wire = stack_build.output_wires[i].clone();
            let mux_out = self.evaluate_mux(&c0_output[i], &c1_output[i], &seed, &stack.muxes[i]);
            known_wires.insert(output_wire.wire_id().clone(), mux_out.clone());
            result_wires.push(mux_out.clone());
        }
        result_wires
    }

    fn evaluate_subcircuit(&mut self, input_wires: Vec<BigUint>, subcircuit_tables: Vec<Vec<BigUint>>, subcircuit_build : &SubcircuitBuild) -> Vec<BigUint> {
        let mut evaluator = HalfGatesEvaluator::new(); // When evaluating subcircuits we reset gate_index to zero so it matches when garbler uses the method, therefore we make a new evaluator. 
        let mut known_wires : HashMap<BigUint, BigUint> = HashMap::new();
        for i in 0..input_wires.len() {
            known_wires.insert(subcircuit_build.input_wires[i].wire_id().clone(), input_wires[i].clone());
        }
        let mut output = vec![];
        for (index, build) in subcircuit_build.builds.iter().enumerate() {
            match build.get_type() {
                BuildType::Gate => {
                    let gate = build.unwrap_to_gate();
                    let wi = known_wires.get(&gate.wi().wire_id()).unwrap().clone();
                    let wj = known_wires.get(&gate.wj().wire_id()).unwrap().clone();
                    let result = evaluator.evaluate_gate(&wi, &wj, &gate.gate_type, &subcircuit_tables[index]);
                    let output_wire_id = gate.wo().wire_id().clone();
                    known_wires.insert(output_wire_id, result.clone());
                    if subcircuit_build.output_wires.contains(gate.wo()) {
                        output.push(result);
                    }
                }
                BuildType::Stack => {
                    let stack_build = build.unwrap_to_stack();
                    // Refactor to avoid having a subcircuit method. Just have a evaluate build method.
                    // let stack_output_wires = self.evaluate_stack(stack_build, stack, &mut known_wires);
                    // for i in 0..stack_output_wires.len() {
                    //     if subcircuit_build.output_wires.contains(&stack_build.output_wires[i]) {
                    //         output.push(stack_output_wires[i].clone());
                    //     }
                    // }
                }
            }
        }
        output
    }

    fn interpret_result(result_wires: Vec<BigUint>, output_conversion: &Vec<[(BigUint, u8); 2]>) -> u32 {
        let mut result : u32 = 0;
        for (index, result_wire) in result_wires.iter().enumerate() {
            if output_conversion[index][1].0 == *result_wire {
                result += 2u32.pow(index as u32);
            } else {
                if !(output_conversion[index][0].0 == *result_wire) {
                    panic!("NO VALID WIRE IN CONVERSION TABLE AT INDEX {}", index)
                }
            }
        }
        result
    }

    fn create_circuit_input(
        &self,
        input: &BigUint,
        required_bits: u64,
    ) -> (VecDeque<[PublicKey; 2]>, Vec<(SecretKey, u8)>) {
        let mut input_choices = VecDeque::new();
        let mut decrypt_choices = vec![];
        for i in 0..required_bits {
            let keypair_real = eg_elliptic::RealKeyPair::new();
            let pk_real = keypair_real.get_pk();
            let sk_real = keypair_real.get_sk();
            let keypair_oblivious = eg_elliptic::ObliviousKeyPair::new();
            let pk_obl = keypair_oblivious.get_pk();
            let bit = input.bit(i) as u8;
            let choice;
            let decrypt_choice;
            if bit == 0 {
                choice = [pk_real.clone(), pk_obl.clone()];
                decrypt_choice = (sk_real.clone(), 0 as u8);
            } else {
                choice = [pk_obl.clone(), pk_real.clone()];
                decrypt_choice = (sk_real.clone(), 1 as u8);
            }
            input_choices.push_back(choice);
            decrypt_choices.push(decrypt_choice);
        }

        (input_choices, decrypt_choices)
    }

    fn evaluate_demux(&mut self, w: &BigUint, seed: &BigUint, demux: &Vec<BigUint>) -> (BigUint, BigUint) {
        let pos = get_position(w, seed);
        let key = gc_kdf(w, seed, &self.get_index());
        self.increment_index();
        let output = key ^ &demux[pos];
        let output_bytes = output.to_bytes_be();
        let c0_wire  = BigUint::from_bytes_be(&output_bytes[..16]);  // first 128 bits
        let c1_wire = BigUint::from_bytes_be(&output_bytes[16..]);  // last 128 bits
        (c0_wire, c1_wire)
    }

    fn evaluate_mux(&mut self, wi: &BigUint, wj: &BigUint, seed: &BigUint, mux: &Vec<BigUint>) -> BigUint {
        let pos = get_mux_pos(seed, wi, wj);
        let key = gc_kdf_128(wi, wj, &self.get_index());
        self.increment_index();
        key ^ &mux[pos]
    }

    fn unstack_material(&self, xor_material_seed: &BigUint, m_cond: &Vec<Vec<BigUint>>, xor_material: &SubcircuitBuild) -> Vec<Vec<BigUint>> {
        let gate_gen = HalfGatesGateGen::new_with_seed(xor_material_seed);
        let mut garbler = Garbler::new(gate_gen);
        let (_, material, _) = garbler.generate_subcircuit(xor_material_seed, xor_material);
        assert_eq!(material.len(), m_cond.len());
        
        let mut unstacked_material = vec![];
        let longest_material = max(m_cond.len(), material.len());
        for table_index in 0..longest_material {
            let m_is_within_index = table_index < material.len();
            let mc_is_within_index = table_index < m_cond.len();
            let mut unstacked_table = vec![];
            if (mc_is_within_index && m_cond[table_index].is_empty()) && (table_index < material.len() && material[table_index].is_empty()) {
                unstacked_table = Vec::new();
                unstacked_material.push(unstacked_table);
                continue;
            }
            for entry_index in 0..2 {
                let mut unstacked_entry = BigUint::ZERO;
                if (mc_is_within_index && m_cond[table_index].is_empty()) && (m_is_within_index && material[table_index].len() > 0) {
                    unstacked_entry = material[table_index][entry_index].clone() // generated material is the longest path, we simply insert it
                } 
                if (m_is_within_index && material[table_index].is_empty()) && (mc_is_within_index && m_cond[table_index].len() > 0) {
                    unstacked_entry = m_cond[table_index][entry_index].clone() // m_cond is the longest path, we simply insert it
                } 
                if (m_is_within_index && material[table_index].len() > 0) && (m_is_within_index && m_cond[table_index].len() > 0) {
                    unstacked_entry = m_cond[table_index][entry_index].clone() ^ material[table_index][entry_index].clone(); // xor both values to stack
                }
                unstacked_table.push(unstacked_entry);
            } 
            unstacked_material.push(unstacked_table);
        }
        unstacked_material
    }

    fn increment_index(&mut self);
    fn get_index(&self) -> &BigUint;
}

fn get_mux_pos(seed: &BigUint, c0_wire: &BigUint, c1_wire: &BigUint) -> usize {
    let s = seed.bit(0) as usize;
    let i = c0_wire.bit(0) as usize;
    let e = c1_wire.bit(0) as usize;
    s * 2 + i ^ e
}

fn get_position(wi: &BigUint, wj: &BigUint) -> usize {
    let l = wi.bit(0) as usize;
    let r = wj.bit(0) as usize;
    l * 2 + r
}
