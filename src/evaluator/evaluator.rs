use k256::{PublicKey, SecretKey};
use num_bigint::BigUint;
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
        wires_i: &Vec<BigUint>,
        wires_j: &Vec<(CipherText, CipherText)>,
        eval_keys: Vec<(SecretKey, u8)>,
        conversion_table: &[(BigUint, u8); 2],
    ) -> u8 {
        let mut outputs: HashMap<&BigUint, BigUint> = HashMap::new(); // id, wire
        let mut circuit_result = 3; // need to return circuit result in a better way without init it

        // Insert constant values
        outputs.insert(&circuit.true_constant_id, circuit.true_constant.clone());
        outputs.insert(&circuit.false_constant_id, circuit.false_constant.clone());

        let mut gate_index = 0;
        for gate in &circuit.gates {
            let wi;
            let wj;
            if gate.is_input_gate {
                wj = decrypt_wj_input(&eval_keys, wires_j, gate_index);
                wi = wires_i[outputs.len() - 2].clone();
            } else {
                // It should already have been calculated and kept in map
                wi = outputs.get(&gate.wi_id).unwrap().clone();
                wj = outputs.get(&gate.wj_id).unwrap().clone();
            }

            let result = self.evaluate_gate(&wi, &wj, &gate.gate_type, &gate.table);
            outputs.insert(&gate.gate_id, result.clone());
            // If last gate, get output
            if gate == &circuit.gates[circuit.gates.len() - 1] {
                println!("conversion_table: {:?}", conversion_table);
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

fn decrypt_wj_input(
    eval_keys: &Vec<(SecretKey, u8)>,
    wires_j: &Vec<(CipherText, CipherText)>,
    gate_index: usize,
) -> BigUint {
    let secret_key = eval_keys[gate_index].0.clone();
    let bit_choice = &eval_keys[gate_index].1;
    let ct = &wires_j[gate_index];
    let wj_ct = match bit_choice {
        0 => &ct.0,
        1 => &ct.1,
        _ => panic!("Invalid bit value: must be 0 or 1"),
    };
    let wj = eg_elliptic::decrypt(&secret_key, wj_ct);
    wj
}
