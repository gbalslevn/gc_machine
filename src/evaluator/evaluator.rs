use k256::{PublicKey, SecretKey};
use num_bigint::{BigUint, ToBigUint};
use std::collections::HashMap;

use crate::{
    gates::gate_gen::GateType, ot::eg_elliptic::{self, CipherText},
};
use crate::circuit_builder::{CircuitBuild, WireBuild};

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
        garbled_gates: &Vec<Vec<BigUint>>,
        constant_wires: &Vec<BigUint>,
        garbler_input: &HashMap<BigUint, BigUint>,
        evaluator_input: &HashMap<BigUint, (CipherText, CipherText)>,
        secret_keys: Vec<(SecretKey, u8)>,
        new_conversion_table: Vec<[(BigUint, u8); 2]>
    ) -> u32 {
        let mut known_wires: HashMap<BigUint, BigUint> = HashMap::new(); // id, wire
        let mut result_wires: Vec<BigUint> = Vec::new();

        // Insert constant values
        known_wires.insert(0.to_biguint().unwrap(), constant_wires[0].to_biguint().unwrap());
        known_wires.insert(1.to_biguint().unwrap(), constant_wires[1].to_biguint().unwrap());

        // Insert garblers input wires
        let garbler_hash_keys = garbler_input.keys().collect::<Vec<_>>();
        for wire_id in garbler_hash_keys {
            let wire = garbler_input.get(wire_id);
            known_wires.insert(wire_id.clone(), wire.unwrap().clone());
        }
        // Insert evaluator wires
        let mut evaluator_hash_keys = evaluator_input.keys().collect::<Vec<_>>();
        evaluator_hash_keys.sort();
        let mut secret_keys_iterator = 0;
        for  key in evaluator_hash_keys {
            let evaluator_ciphers = evaluator_input.get(key).unwrap();
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

        // Evaluate each gate
        for (index, gate) in circuit_build.gates.iter().enumerate() {
            let wi;
            let wj;
            wi = known_wires.get(&gate.wi().wire_id()).unwrap().clone();
            wj = known_wires.get(&gate.wj().wire_id()).unwrap().clone();
            let result = self.evaluate_gate(&wi, &wj, &gate.gate_type, &garbled_gates[index]);
            known_wires.insert(gate.wo().wire_id().clone(), result.clone());

            // Store all result wires
            if gate.wo().ready_at_layer() == &circuit_build.output_layer {
                result_wires.push(result.clone());
            }
        }

        Self::interpret_result(result_wires, &new_conversion_table)
    }

    fn interpret_result(result_wires: Vec<BigUint>, output_conversion: &Vec<[(BigUint, u8); 2]>) -> u32 {
        let mut result: u32 = 0;
        for (index, result_wire) in result_wires.iter().enumerate() {
            if output_conversion[index][1].0 == *result_wire {
                result += 2u32.pow(index as u32);
            }
        }
        result
    }

    fn create_circuit_input(
        &self,
        input: &BigUint,
        required_bits: u64,
    ) -> (Vec<[PublicKey; 2]>, Vec<(SecretKey, u8)>) {
        let mut input_choices = vec![];
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
            input_choices.push(choice);
            decrypt_choices.push(decrypt_choice);
        }

        (input_choices, decrypt_choices)
    }
    fn increment_index(&mut self);
    fn get_index(&self) -> &BigUint;
}
