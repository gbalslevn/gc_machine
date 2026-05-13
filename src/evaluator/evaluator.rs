use k256::{PublicKey, SecretKey};
use num_bigint::{BigUint, ToBigUint};
use std::{cmp::max, collections::{HashMap, VecDeque}};

use crate::{
    circuit_builder::{Build, BuildCount, BuildType, StackBuild, SubcircuitBuild}, crypto_utils::{gc_kdf, gc_kdf_128}, evaluator::{self, half_gates_evaluator::HalfGatesEvaluator}, garbler::{Circuit, Garbler, Stack}, gates::{gate_gen::{GateGen, GateType}, half_gates_gate_gen::HalfGatesGateGen}, ot::eg_elliptic::{self}, wires::wire_gen::{Wire, WireGen},
};
use crate::circuit_builder::{CircuitBuild};

pub trait Evaluator: Sized {
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
        evaluate_builds(&circuit_build.builds, &circuit.material, &mut known_wires, self);
        let mut result_wires: Vec<BigUint> = Vec::new();
        let mut result_wires_id: Vec<BigUint> = Vec::new();
        for output_wire in &circuit_build.output_wires {
            let output = known_wires.get(output_wire.wire_id()).unwrap().clone();
            result_wires_id.push(output_wire.wire_id().clone());
            result_wires.push(output);
        }
        interpret_result(result_wires, &circuit.output_conversion, &result_wires_id)
    }

    fn evaluate_stack(&mut self, stack_build : &StackBuild, stack : &Stack, known_wires : &mut HashMap<BigUint, BigUint>) -> Vec<BigUint> {
        let mut result_wires = vec![];
        let seed = known_wires.get(stack_build.conditional.wire_id()).unwrap().clone();
        let c0 = self.unstack_material(&seed, &stack.m_cond, &stack_build.c1_circuit, &stack_build.c0_circuit);
        let c1 = self.unstack_material(&seed, &stack.m_cond, &stack_build.c0_circuit, &stack_build.c1_circuit);
        
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
        for (i, (c0w, c1w)) in c0_output.iter().zip(c1_output.iter()).enumerate() {
            println!("output {}: c0={} c1={}", i, c0w, c1w);
        }

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
        
        evaluate_builds(&subcircuit_build.builds, &subcircuit_tables, &mut known_wires, &mut evaluator);
        // Collect output
        let mut output = vec![];
        for wire_build in &subcircuit_build.output_wires {
            let wire = known_wires.get(wire_build.wire_id()).unwrap();
            output.push(wire.clone());
        }
        // pad the output
        let padding_needed = subcircuit_build.output_wires.len() - output.len();
        for i in 0..padding_needed {
            let false_constant_wire_id = 0.to_biguint().unwrap();
            output.push(known_wires.get(&false_constant_wire_id).unwrap().clone());
        }
        output
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
        let mut buf = [0u8; 32];
        let bytes = output.to_bytes_be();
        buf[32 - bytes.len()..].copy_from_slice(&bytes);
        let c0_wire = BigUint::from_bytes_be(&buf[..16]); // first 128 bits
        let c1_wire = BigUint::from_bytes_be(&buf[16..]); // last 128 bits  
        (c0_wire, c1_wire)
    }

    fn evaluate_mux(&mut self, wi: &BigUint, wj: &BigUint, seed: &BigUint, mux: &Vec<BigUint>) -> BigUint {
        let pos = get_mux_pos(seed, wi, wj);
        let key = gc_kdf_128(wi, wj, &self.get_index());
        self.increment_index();
        key ^ &mux[pos]
    }

    fn unstack_material(&self, seed: &BigUint, m_cond: &Vec<Vec<BigUint>>, build_to_generate: &SubcircuitBuild, unstacked_build : &SubcircuitBuild) -> Vec<Vec<BigUint>> {
        let gate_gen = HalfGatesGateGen::new_with_seed(seed); 
        let mut garbler = Garbler::new(gate_gen);
        let (_, material, _) = garbler.generate_subcircuit(seed, build_to_generate); // seems weird we need to provide seed to a garbler which has been init by that seed
        let mut material_gen = HalfGatesGateGen::new_with_seed(seed);
        // fill in material instead of empty table representing FreeXOR gates
        let mut filled_material = vec![];
        for table in material {
            if table.len() > 0 {
                filled_material.push(table);
            } else {
                let material = material_gen.get_wire_gen().generate_input_wire();
                let table = vec![material.w0().clone(), material.w1().clone()];
                filled_material.push(table);
            }
        }
        
        let mut unstacked_material = vec![];
        // Pad generated material if neccesary 
        let padding = vec![0.to_biguint().unwrap(), 0.to_biguint().unwrap()];    
        if filled_material.len() < m_cond.len() {
            for i in filled_material.len()..m_cond.len() {
                filled_material.push(padding.clone());
            }
        }
        for table_index in 0..m_cond.len() {
            let mut unstacked_table = vec![];
            for entry_index in 0..2 {
                let unstacked_entry = m_cond[table_index][entry_index].clone() ^ filled_material[table_index][entry_index].clone(); // xor both values to stack
                unstacked_table.push(unstacked_entry);
            } 
            unstacked_material.push(unstacked_table);
        }
        // insert FreeXOR gates again
        let mut unstacked_with_xor_gates: Vec<Vec<BigUint>> = vec![];
        for build in &unstacked_build.builds {
            match build.get_type() {
                BuildType::Gate => {
                    let gate_build = build.unwrap_to_gate();
                    if gate_build.gate_type() == &GateType::XOR || gate_build.gate_type() == &GateType::XNOR {
                        unstacked_material.remove(0);
                        unstacked_with_xor_gates.push(vec![]);
                    } else {
                        unstacked_with_xor_gates.push(unstacked_material.remove(0));
                    }
                }   
                BuildType::Stack => {
                    let stack_build = build.unwrap_to_stack();
                    let amount_material_in_stack = stack_build.input_wires.len() * 4 + stack_build.m_cond_len + stack_build.output_wires.len() * 2; // demux material + mux material + subcircuit material. 
                    let stack_material: Vec<Vec<BigUint>> = unstacked_material.drain(0..amount_material_in_stack).collect(); 
                    unstacked_with_xor_gates.extend(stack_material);
                }             
            }
        }
        unstacked_with_xor_gates
    }

    fn increment_index(&mut self);
    fn get_index(&self) -> &BigUint;
}

fn evaluate_builds<E : Evaluator>(builds: &Vec<Build>, build_material: &Vec<Vec<BigUint>>, known_wires: &mut HashMap<BigUint, BigUint>, evaluator: &mut E) {
    let mut material_iter = build_material.iter();
    for build in builds {
        match build.get_type() {
            BuildType::Gate => {
                let gate_build = build.unwrap_to_gate();
                let gate_material = material_iter.next().unwrap();
                let wi = known_wires.get(&gate_build.wi().wire_id()).unwrap().clone();
                let wj = known_wires.get(&gate_build.wj().wire_id()).unwrap().clone();
                let result = evaluator.evaluate_gate(&wi, &wj, &gate_build.gate_type, &gate_material);
                let output_wire_id = gate_build.wo().wire_id().clone();
                known_wires.insert(output_wire_id, result.clone());
            }
            BuildType::Stack => {
                let stack_build = build.unwrap_to_stack();
                let stack_material_len = (stack_build.input_wires.len() * 4) + stack_build.m_cond_len + (stack_build.output_wires.len() * 2);
                let mut stack_material = vec![];
                for i in 0..stack_material_len {
                    stack_material.push(material_iter.next().unwrap().clone())
                }
                let stack = get_stack_from_material( &stack_material, stack_build); // perhaps provide the iter instead of extracting material here
                evaluator.evaluate_stack(stack_build, &stack, known_wires);
            }
        }
    }
}

fn interpret_result(result_wires: Vec<BigUint>, output_conversion: &Vec<[(BigUint, u8); 2]>, result_wires_id: &Vec<BigUint>) -> u32 {
    let mut result : u32 = 0;
    for (index, result_wire) in result_wires.iter().enumerate() {
        if output_conversion[index][1].0 == *result_wire {
            result += 2u32.pow(index as u32);
            println!("Found output wire with bit 1 and id {}", result_wires_id[index]);
        } else {
            if !(output_conversion[index][0].0 == *result_wire) {
                // panic!("NO VALID WIRE IN CONVERSION TABLE FOR WIRE WITH ID {}", result_wires_id[index])
                println!("NO VALID WIRE IN CONVERSION TABLE FOR WIRE WITH ID {}", result_wires_id[index])
            } else {
                println!("Found output wire with bit 0 and id {}", result_wires_id[index]);
            }
        }
    }
    result
}

fn get_mux_pos(seed: &BigUint, c0_wire: &BigUint, c1_wire: &BigUint) -> usize {
    let s = seed.bit(0) as usize;
    let i = c0_wire.bit(0) as usize;
    let e = c1_wire.bit(0) as usize;
    s * 2 + (i ^ e)
}

fn get_position(wi: &BigUint, wj: &BigUint) -> usize {
    let l = wi.bit(0) as usize;
    let r = wj.bit(0) as usize;
    l * 2 + r
}

fn get_stack_from_material(stack_material: &Vec<Vec<BigUint>>, stack_build: &StackBuild) -> Stack {
    let mut material = stack_material.clone();
    // We retrive demuxes from received material
    let mut demuxes = vec![];
    for _ in &stack_build.input_wires {
        // 1 demux is 2 tables as it has 4 entries but each entry is 256 bits, so actually its 4 entries, which means we need 4 tables. 
        let demux_material: Vec<Vec<BigUint>> = material.drain(0..4).collect(); 
        // Combine the two 128 bit c_0, c_1 input labels for each 4 entries so we can evaluate the demux
        let entry_0 = &demux_material[0][0] << 128 | &demux_material[0][1];
        let entry_1 = &demux_material[1][0] << 128 | &demux_material[1][1];
        let entry_2 = &demux_material[2][0] << 128 | &demux_material[2][1];
        let entry_3 = &demux_material[3][0] << 128 | &demux_material[3][1];
        let demux = vec![entry_0, entry_1, entry_2, entry_3];
        demuxes.push(demux);
    }
    // We retrive m_cond from material
    let m_cond = material.drain(0..stack_build.m_cond_len).collect();    
    
    // We retrive muxes from material
    let mut muxes = vec![];
    for _ in &stack_build.output_wires {
        let mux_material: Vec<Vec<BigUint>> = material.drain(0..2).collect(); // 1 mux is 2 tables as it has 4 entries.
        let mux = vec![mux_material[0][0].clone(), mux_material[0][1].clone(), mux_material[1][0].clone(), mux_material[1][1].clone()];
        muxes.push(mux);
    }

    Stack {demuxes, m_cond, muxes}
}
