use std::{collections::HashMap};
use std::collections::VecDeque;
use k256::PublicKey;
use num_bigint::{BigUint, ToBigUint};
use rand_chacha::ChaCha20Rng;

use crate::{
    circuit_builder::CircuitBuild, gates::gate_gen::{GateGen}, ot::eg_elliptic::{self, CipherText}, wires::wire_gen::{Wire, WireGen}
};

pub struct Garbler<G: GateGen<W>, W: WireGen> {
    gate_gen: G,
    wire_gen: W,
}

impl<G: GateGen<W>, W: WireGen> Garbler<G, W> {
    pub fn new(gate: G, wire: W) -> Self {
        Self {
            gate_gen: gate,
            wire_gen: wire,
        }
    }
    pub fn create_circuit(
        &mut self,
        circuit_build: &CircuitBuild,
        garblers_input_choices: &mut VecDeque<u8>,
        mut evaluators_input_choices: VecDeque<[PublicKey; 2]>,
    ) -> (
        Vec<Vec<BigUint>>, // Ciphertexts
        Vec<BigUint>, // Constant wires
        HashMap<BigUint, BigUint>, // Garbler input wires
        HashMap<BigUint, (CipherText, CipherText)>, // Evaluator input wires
        Vec<[(BigUint, u8); 2]> // Output conversion table
    ) {
        let mut garbled_gates: Vec<Vec<BigUint>> = Vec::new();
        let mut constant_wires: Vec<BigUint> = vec![];
        let mut garbler_inputs: HashMap<BigUint, BigUint> = HashMap::new();
        let mut evaluator_inputs: HashMap<BigUint, (CipherText, CipherText)> = HashMap::new();
        let mut known_wires: HashMap<BigUint, Wire> = HashMap::new();
        let mut wi;
        let mut wj;
        let mut new_output_conversion: Vec<[(BigUint, u8); 2]> = Vec::new();
        let gates = circuit_build.get_gates();

        // insert constants for true and false wire into known_wires, to enable eg. NOT gates
        self.insert_constant_wires(&mut known_wires, &mut constant_wires);
        let mut rng = self.wire_gen.get_rng().clone();
        for (gate_index, gate) in gates.iter().enumerate() {
            let gate_is_input_layer = gate.wo().ready_at_layer() == &1;
            if gate_is_input_layer {
                // Generate wires if not already generated (copied wires are already generated)
                let wi_id = gate.wi().wire_id().clone();
                let wj_id = gate.wj().wire_id().clone();
                let wi_is_new_wire = !garbler_inputs.contains_key(&wi_id);
                let wj_is_new_wire = !evaluator_inputs.contains_key(&wj_id);
                if wi_is_new_wire {
                    wi = self.wire_gen.generate_input_wire();
                    known_wires.insert(wi_id.clone(), wi.clone());
                    let garbler_input_choice = garblers_input_choices.pop_front().unwrap();
                    let selected_wire = match garbler_input_choice {
                        0 => wi.w0(),
                        1 => wi.w1(),
                        _ => panic!("Invalid bit value: must be 0 or 1"),
                    };
                    garbler_inputs.insert(wi_id.clone(), selected_wire.clone());
                }
                if wj_is_new_wire {
                    wj = self.wire_gen.generate_input_wire();
                    known_wires.insert(wj_id.clone(), wj.clone());
                    // Encrypt with received publickeys from OT. The real and the oblivious
                    let wj_encrypted =
                        self.gen_encrypted_wire(&wj, &evaluators_input_choices.pop_front().unwrap(), &mut rng);
                    evaluator_inputs.insert(wj_id.clone(), wj_encrypted.clone());

                }
            }

            wi = known_wires.get(&gate.wi().wire_id()).unwrap().clone();
            wj = known_wires.get(&gate.wj().wire_id()).unwrap().clone();


            let new_gate = self.gate_gen.generate_gate(
                gate.gate_type().clone(),
                wi.clone(),
                wj.clone()
            );

            let output_wire_id = gate.wo().wire_id().clone();
            known_wires.insert(output_wire_id.clone(), new_gate.wo.clone());
            let table = new_gate.to_table();
            
            // Put all output wires in to the output_conversion table
            if circuit_build.output_wires.contains(gate.wo()) {
                new_output_conversion.push([(new_gate.wo.w0().clone(), 0), (new_gate.wo.w1().clone(), 1)]);
            }

            
            // Store the ciphertexts for the gate
            garbled_gates.push(table);
        }
        (garbled_gates, constant_wires, garbler_inputs, evaluator_inputs, new_output_conversion)
    }

    pub fn create_circuit_input(&self, input: &BigUint, required_bits: u64) -> VecDeque<u8> {
        let mut list = VecDeque::new();;
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

    fn insert_constant_wires(&mut self, known_wires: &mut HashMap<BigUint, Wire>, constant_wires: &mut Vec<BigUint>) {
        let true_constant = self.wire_gen.generate_input_wire();
        let false_constant = self.wire_gen.generate_input_wire();
        known_wires.insert(
            0.to_biguint().unwrap(),
            false_constant.clone(),
        );
        known_wires.insert(
            1.to_biguint().unwrap(),
            true_constant.clone(),
        );
        constant_wires.insert(0, false_constant.w0().clone());
        constant_wires.insert(1, true_constant.w1().clone());
    }

    // Encrypts a wire for the evaluator as a part of OT
    fn gen_encrypted_wire(
        &self,
        wire: &Wire,
        input_choice: &[PublicKey; 2],
        rng : &mut ChaCha20Rng
    ) -> (CipherText, CipherText) {
        let pk_0 = &input_choice[0];
        let wj_0_ct = eg_elliptic::encrypt(rng, pk_0, wire.w0());
        let pk_1 = &input_choice[1];
        let wj_1_ct = eg_elliptic::encrypt(rng, pk_1, wire.w1());

        let wj_encrypted = (wj_0_ct, wj_1_ct);
        wj_encrypted
    }
}
