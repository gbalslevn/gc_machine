use k256::{PublicKey, SecretKey};
use num_bigint::{BigUint, ToBigUint};
use std::{cmp::max, collections::{HashMap, VecDeque}};

use crate::{
    circuit_builder::{BuildType, SubcircuitBuild}, crypto_utils::{gc_kdf, gc_kdf_128}, evaluator::{self, half_gates_evaluator::HalfGatesEvaluator}, garbler::Circuit, gates::{gate_gen::{GateGen, GateType}, half_gates_gate_gen::HalfGatesGateGen}, ot::eg_elliptic::{self}, wires::wire_gen::{Wire, WireGen}
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
        for (index, build) in circuit_build.builds.iter().enumerate() {
            match build.get_type() {
                BuildType::Gate => {
                    let gate = build.unwrap_to_gate();
                    let wi = known_wires.get(&gate.wi().wire_id()).unwrap().clone();
                    let wj = known_wires.get(&gate.wj().wire_id()).unwrap().clone();
                    let result = self.evaluate_gate(&wi, &wj, &gate.gate_type, &circuit.gates[index]); 
                    known_wires.insert(gate.wo().wire_id().clone(), result.clone());
        
                    // Store all result wires
                    if circuit_build.output_wires.contains(gate.wo()) {
                        result_wires.push(result.clone());
                    }
                }
                BuildType::Stack => {
                    let stack_build = build.unwrap_to_stack();
                    let stack = circuit.stacks.get(&stack_build.id.to_biguint().unwrap()).unwrap();
                    let seed = known_wires.get(stack_build.conditional.wire_id()).unwrap().clone();
                    let c0= unstack_material(&seed, &stack.m_cond, &stack_build.c0_circuit);
                    let c1 = unstack_material(&seed, &stack.m_cond, &stack_build.c1_circuit);
                    
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
                        if stack_build.output_wires.contains(&output_wire) {
                            result_wires.push(mux_out.clone());
                        }
                    }
                }
            }
        }

        Self::interpret_result(result_wires, &circuit.output_conversion)
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
                    if output.len() < subcircuit_build.output_wires.len() {

                    }
                }
                BuildType::Stack => {

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

    fn increment_index(&mut self);
    fn get_index(&self) -> &BigUint;
}

fn get_mux_pos(seed: &BigUint, c0_wire: &BigUint, c1_wire: &BigUint) -> usize {
    let s = seed.bit(0) as usize;
    let i = c0_wire.bit(0) as usize;
    let e = c1_wire.bit(0) as usize;
    s * 4 + i * 2 + e
}

fn get_position(wi: &BigUint, wj: &BigUint) -> usize {
    let l = wi.bit(0) as usize;
    let r = wj.bit(0) as usize;
    l * 2 + r
}

pub fn unstack_material(xor_material_seed: &BigUint, m_cond: &Vec<Vec<BigUint>>, xor_material: &SubcircuitBuild) -> Vec<Vec<BigUint>> {
    let material = generate_subcircuit(xor_material_seed, xor_material);
    
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

fn generate_subcircuit(seed: &BigUint, subcircuit_build: &SubcircuitBuild) -> Vec<Vec<BigUint>> {
    let mut gate_gen = HalfGatesGateGen::new_with_seed(seed);

    let mut known_wires: HashMap<BigUint, Wire> = HashMap::new();
    let input_wires = &subcircuit_build.input_wires;
    
    // insert all input wires
    for input_wire in input_wires {
        let wire = gate_gen.get_wire_gen().generate_input_wire();
        known_wires.insert(input_wire.wire_id().clone(), wire);
    }

    let builds = &subcircuit_build.builds;
    let mut subcircuit: Vec<Vec<BigUint>> = vec![];
    for build in builds {
        match build.get_type() {
            BuildType::Gate => {
                let gate = build.unwrap_to_gate();
                let wi = known_wires.get(&gate.wi().wire_id()).unwrap().clone();
                let wj = known_wires.get(&gate.wj().wire_id()).unwrap().clone();
    
                let new_gate = gate_gen.generate_gate(
                    gate.gate_type().clone(),
                    wi.clone(),
                    wj.clone()
                );
                let output_wire_id = gate.wo().wire_id();
                known_wires.insert(output_wire_id.clone(), new_gate.wo.clone());
                let table = new_gate.to_table();
    
                // Store the ciphertexts for the gate
                subcircuit.push(table);
            }
            BuildType::Stack => {
                todo!("Handle if there is a nested stack")
            }
        }
    }
    subcircuit
}
