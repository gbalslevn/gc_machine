use k256::{PublicKey, SecretKey};
use num_bigint::{BigUint, ToBigUint};
use std::collections::HashMap;

use crate::{
    garbler::CircuitEval,
    gates::gate_gen::GateType, ot::eg_elliptic::{self, CipherText},
};

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
            GateType::XOR => self.evaluate_xor_gate(wi, wj, table),
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
        circuit: &CircuitEval,
        garbler_input: &HashMap<BigUint, BigUint>,
        evaluator_input: &HashMap<BigUint, (CipherText, CipherText)>,
        secret_keys: Vec<(SecretKey, u8)>,
        conversion_table: &[(BigUint, u8); 2],
    ) -> u8 {
        let mut outputs: HashMap<BigUint, BigUint> = HashMap::new(); // id, wire
        let mut circuit_result = 3; // need to return circuit result in a better way without init it

        // Insert constant values
        outputs.insert(circuit.true_constant_id.clone(), circuit.true_constant.clone());
        outputs.insert(circuit.false_constant_id.clone(), circuit.false_constant.clone());
        // Insert garblers input wires
        let garbler_hash_keys = garbler_input.keys().collect::<Vec<_>>();
        for key in garbler_hash_keys {
            let wire = garbler_input.get(key);
            if key.clone() == 2.to_biguint().unwrap() {
            }
            outputs.insert(key.clone(), wire.unwrap().clone());
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
            outputs.insert(key.clone(), wire.clone());
            if key.clone() == 4.to_biguint().unwrap() {
            }
            secret_keys_iterator += 1;
        }

        let mut gate_index = 0;
        for (index, gate) in circuit.gates.iter().enumerate() {
            let wi;
            let wj;


            wi = outputs.get(&gate.wi_id).unwrap().clone();
            wj = outputs.get(&gate.wj_id).unwrap().clone();

            let result = self.evaluate_gate(&wi, &wj, &gate.gate_type, &gate.table);
            outputs.insert(gate.output_wire_id.clone(), result.clone());
            // If last gate, get output
            if gate == &circuit.gates[circuit.gates.len() - 1] {
                if result == conversion_table[0].0 {
                    circuit_result = conversion_table[0].1;
                }
                if result == conversion_table[1].0 {
                    circuit_result = conversion_table[1].1;
                }
            }
            gate_index += 1;
        }
        circuit_result
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
