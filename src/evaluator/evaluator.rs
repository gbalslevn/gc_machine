use num_bigint::{BigUint, ToBigUint};
use std::collections::HashMap;

use crate::{
    gates::gates::{Gate, GateType},
    ot::ot::{self, CipherText, PublicParameters, SecretKey},
    wires::wires::Wire,
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
        circuit: &Vec<Gate>,
        wires_i: &Vec<BigUint>,
        wires_j: &Vec<(CipherText, CipherText)>,
        conversion_table: &[(BigUint, u8); 2],
        keys: &Vec<((&SecretKey, &PublicParameters), u8)>,
    ) -> u8 {
        // Perhaps make a input gate struct which differs from a normal gate struct. Make it easier to access the wires where we dont have to send a list of wi's also. Also remove the notion of wires, as it reveals which label is w0 and w1, just use 4 labels
        let mut gate_index = 0;
        // let mut outputs: HashMap<&BigUint, BigUint> = HashMap::new(); // id, wire
        let mut outputs: Vec<BigUint> = vec![];
        let mut wi = BigUint::ZERO;
        let mut wj = BigUint::ZERO;
        let mut circuit_result = 3; // need to return circuit result in a better way without init it

        for gate in circuit {
            let is_input_gate = &wires_i[gate_index] == gate.wi.w0() || &wires_i[gate_index] == gate.wi.w1();
            if true {
                let bit_choice = &keys[gate_index].1;
                let secret_key = keys[gate_index].0.0.clone();
                let pp = keys[gate_index].0.1;
                let wj_ct = match bit_choice {
                    0 => {
                        &wires_j[gate_index].0
                    },
                    1 => {
                        &wires_j[gate_index].1
                    },
                    _ => panic!("Invalid bit value: must be 0 or 1"),
                };
                wj = ot::decrypt(&pp, secret_key, wj_ct.clone());
                wi = wires_i[gate_index].clone();
            } else {
                // It should already have been calculated
                // Oh no this is not good code. is it okay that we call a wire with w0? Then the eval would know it is the wire representing 0.... There is also some naming situation. Should we call something a wire when it only contains BigUint, or should we call it label.
                // Should provide wires with its producer id and get from a hashmap. For now we just search in list.
                // wi = outputs
                //     .get(&gate.wi.)
                //     .or(outputs.get(&gate.wi.w1()))
                //     .unwrap().clone();
                // wj = outputs
                //     .get(&gate.wj.w0())
                //     .or(outputs.get(&gate.wj.w1()))
                //     .unwrap().clone();
                // for label in &outputs {
                //     if label == gate.wi.w0() || label == gate.wi.w1() {
                //         wi = label.clone();
                //     }
                //     if label == gate.wj.w0() || label == gate.wj.w1() {
                //         wj = label.clone();
                //     }
                // }
            }

            // use gate id of wires_j to know when it is an input gate, and therefore when to decrypt
            // println!("gate table is: {:?}", &gate.table);
            // println!("gate wi is: {:?}", &gate.wi);
            // println!("gate wj is: {:?}", &gate.wj);
            // println!("gate w0 is: {:?}", &gate.wo);
            // println!("gate wj is: {:?}", &wj);
            let result = Self::evaluate_gate(&wi, &wj, &gate.gate_type, &gate.gate_id, &gate.table);
            outputs.push(result.clone());
            // outputs.insert(&gate.gate_id, result.clone());
            gate_index += 1;
            // If last gate, get output
            if gate == &circuit[circuit.len() - 1] {
                // println!("Conversion 0 is: {}", conversion_table[0].0);
                // println!("Conversion 1 is: {}", conversion_table[1].0);
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
}
