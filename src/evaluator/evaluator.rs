use num_bigint::BigUint;
use std::collections::HashMap;

use crate::{
    garbler::CircuitEval,
    gates::gates::GateType,
    ot::ot::{self, CipherText, PublicKey, PublicParameters, SecretKey},
};
pub trait Evaluator {
    fn evaluate_gate(
        wi: &BigUint,
        wj: &BigUint,
        gate_type: &GateType,
        gate_id: &BigUint,
        table: &Vec<BigUint>,
    ) -> BigUint {
        match gate_type {
            GateType::AND => Self::evaluate_and_gate(wi, wj, gate_id, table),
            GateType::XOR => Self::evaluate_xor_gate(wi, wj, gate_id, table),
        }
    }

    fn evaluate_and_gate(
        wi: &BigUint,
        wj: &BigUint,
        gate_id: &BigUint,
        table: &Vec<BigUint>,
    ) -> BigUint;
    fn evaluate_xor_gate(
        wi: &BigUint,
        wj: &BigUint,
        gate_id: &BigUint,
        table: &Vec<BigUint>,
    ) -> BigUint;

    fn evaluate_circuit(
        circuit: &CircuitEval,
        wires_i: &Vec<BigUint>,
        wires_j: &Vec<(CipherText, CipherText)>,
        conversion_table: &[(BigUint, u8); 2],
        eval_keys: Vec<(SecretKey, u8)>, pp : &PublicParameters,
    ) -> u8 {
        // Perhaps make a input gate struct which differs from a normal gate struct. Make it easier to access the wires where we dont have to send a list of wi's also. Also remove the notion of wires, as it reveals which label is w0 and w1, just use 4 labels
        let mut gate_index = 0; // need to start at 2 because of the two constants. 
        let mut outputs: HashMap<&BigUint, BigUint> = HashMap::new(); // id, wire
        let mut circuit_result = 3; // need to return circuit result in a better way without init it

        // Insert constant values
        outputs.insert(&circuit.true_constant_id, circuit.true_constant.clone());
        outputs.insert(&circuit.false_constant_id, circuit.false_constant.clone());

        for gate in &circuit.gates {
            let wi;
            let wj;
            if gate.is_input_gate {
                wj = decrypt_wj_input(&eval_keys, pp, wires_j, gate_index);
                wi = wires_i[gate_index].clone();
            } else {
                // It should already have been calculated and kept in map
                wi = outputs.get(&gate.wi_id).unwrap().clone();
                wj = outputs.get(&gate.wj_id).unwrap().clone();
            }

            let result = Self::evaluate_gate(&wi, &wj, &gate.gate_type, &gate.gate_id, &gate.table);
            outputs.insert(&gate.gate_id, result.clone());
            gate_index += 1;
            // If last gate, get output
            if gate == &circuit.gates[circuit.gates.len() - 1] {
                if result == conversion_table[0].0 {
                    circuit_result = conversion_table[0].1;
                }
                if result == conversion_table[1].0 {
                    circuit_result = conversion_table[1].1;
                }
            }
        }
        circuit_result
    }

    fn create_circuit_input(
        input: &BigUint,
        required_bits: u64,
        pp: &PublicParameters,
    ) -> (Vec<[PublicKey; 2]>, Vec<(SecretKey, u8)>) {
        let mut input_choices = vec![];
        let mut decrypt_choices = vec![];
        for i in 0..required_bits {
            let keypair_real = ot::RealKeyPair::new(&pp);
            let pk_real = keypair_real.get_public_key();
            let sk_real = keypair_real.get_secret_key();
            let pk_oblivious = ot::ObliviousKeyPair::new(&pp).get_public_key();
            let bit = input.bit(i) as u8;
            let choice;
            let decrypt_choice;
            if bit == 0 {
                choice = [pk_real.clone(), pk_oblivious.clone()];
                decrypt_choice = (sk_real.clone(), 0 as u8);
            } else {
                choice = [pk_oblivious.clone(), pk_real.clone()];
                decrypt_choice = (sk_real.clone(), 1 as u8);
            }
            input_choices.push(choice);
            decrypt_choices.push(decrypt_choice);
        }

        (input_choices, decrypt_choices)
    }
}

fn decrypt_wj_input(
    eval_keys: &Vec<(SecretKey, u8)>, pp : &PublicParameters,
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
    let wj = ot::decrypt(&pp, &secret_key, wj_ct);
    wj
}
