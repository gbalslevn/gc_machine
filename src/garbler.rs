use std::{collections::HashMap};

use k256::PublicKey;
use num_bigint::{BigUint, ToBigUint};
use rand_chacha::ChaCha20Rng;

use crate::{
    circuit_builder::CircuitBuild, gates::gate_gen::{GateGen, GateType}, ot::eg_elliptic::{self, CipherText}, wires::wire_gen::{Wire, WireGen}
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
        garblers_input_choices: &Vec<u8>,
        evaluators_input_choices: Vec<[PublicKey; 2]>,
    ) -> (
        CircuitEval,
        HashMap<BigUint, BigUint>,
        HashMap<BigUint, (CipherText, CipherText)>,
        [(BigUint, u8); 2],
    ) {
        let mut circuit: Vec<GateEval> = vec![];
        let mut wi_hashmap: HashMap<BigUint, BigUint> = HashMap::new();
        let mut wj_hashmap: HashMap<BigUint, (CipherText, CipherText)> = HashMap::new();
        let mut outputs: HashMap<BigUint, Wire> = HashMap::new();
        // let mut outputs_index = 0;
        let mut wi;
        let mut wj;
        let mut output_conversion: [(BigUint, u8); 2] =
            [(BigUint::from(0u32), 0), (BigUint::from(0u32), 0)];
        let gates = circuit_build.get_gates();

        // insert constants for true and false wire, to enable eg. NOT gates
        let true_constant = self.wire_gen.generate_input_wire();
        let false_constant = self.wire_gen.generate_input_wire();
        outputs.insert(
            circuit_build.get_true_constant().wire_id().clone(),
            true_constant.clone(),
        );
        outputs.insert(
            circuit_build.get_false_constant().wire_id().clone(),
            false_constant.clone(),
        );
        
        let mut gate_index = 0;
        let mut rng = self.wire_gen.get_rng().clone();
        for (index, gate) in gates.iter().enumerate() {
            let gate_is_input_layer = gate.wo().output_layer() == &1.to_biguint().unwrap();
            if gate_is_input_layer {
                // Generate wires if not already generated (copied wires are already generated)
                let wi_id = gate.wi().wire_id().clone();
                let wj_id = gate.wj().wire_id().clone();
                let wi_is_new_wire = !wi_hashmap.contains_key(&wi_id);
                let wj_is_new_wire = !wj_hashmap.contains_key(&wj_id);
                if wi_is_new_wire {
                    wi = self.wire_gen.generate_input_wire();
                    outputs.insert(wi_id.clone(), wi.clone());
                    let garbler_input_choice = garblers_input_choices[gate_index];
                    let selected_wire = match garbler_input_choice {
                        0 => wi.w0(),
                        1 => wi.w1(),
                        _ => panic!("Invalid bit value: must be 0 or 1"),
                    };
                    wi_hashmap.insert(wi_id.clone(), selected_wire.clone());
                }
                if wj_is_new_wire {
                    wj = self.wire_gen.generate_input_wire();
                    outputs.insert(wj_id.clone(), wj.clone());
                    // Encrypt with received publickeys from OT. The real and the oblivious
                    let wj_encrypted =
                        self.gen_encrypted_wire(&wj, &evaluators_input_choices[gate_index], &mut rng);
                    wj_hashmap.insert(wj_id.clone(), wj_encrypted.clone());

                }
            }
            if *gate.wo().wire_id() == 8.to_biguint().unwrap() {
            }

            wi = outputs.get(&gate.wi().wire_id()).unwrap().clone();
            wj = outputs.get(&gate.wj().wire_id()).unwrap().clone();


            let new_gate = self.gate_gen.generate_gate(
                gate.gate_type().clone(),
                wi.clone(),
                wj.clone() // need to +2 as we already have two constant inputs, acting like theyre coming from gateid_0 and 1. Need to make it better
            );

            let output_wire_id = gate.wo().wire_id().clone();
            outputs.insert(output_wire_id.clone(), new_gate.wo.clone());
            let gate_eval = new_gate.to_gate_eval(
                output_wire_id,
                gate.wi().wire_id().clone(),
                gate.wj().wire_id().clone(),
                gate_is_input_layer,
            );
            let is_last_gate = gate == &gates[gates.len() - 1];
            if is_last_gate {
                output_conversion = [(new_gate.wo.w0().clone(), 0), (new_gate.wo.w1().clone(), 1)];
            }

            circuit.push(gate_eval);
            gate_index += 1;
        }
        let circuit = CircuitEval {
            gates: circuit,
            true_constant: true_constant.w1().clone(),
            true_constant_id: circuit_build.get_true_constant().wire_id().clone(),
            false_constant: false_constant.w0().clone(),
            false_constant_id: circuit_build.get_false_constant().wire_id().clone(),
        };
        (circuit, wi_hashmap, wj_hashmap, output_conversion)
    }

    pub fn create_circuit_input(&self, input: &BigUint, required_bits: u64) -> Vec<u8> {
        let mut list = vec![];
        for i in 0..required_bits {
            let bit = input.bit(i) as u8;
            if bit == 0 {
                list.push(0 as u8);
            } else {
                list.push(1 as u8)
            }
        }
        list
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

#[derive(Debug)]
pub struct CircuitEval {
    pub gates: Vec<GateEval>,
    pub true_constant_id: BigUint,
    pub false_constant_id: BigUint,
    pub true_constant: BigUint,
    pub false_constant: BigUint,
}

// Perhaps look into combining Gate and GateEval
#[derive(PartialEq, Debug)]
pub struct GateEval {
    pub output_wire_id: BigUint,
    pub gate_type: GateType,
    pub table: Vec<BigUint>,
    pub wi_id: BigUint, // output wire produced from gate with "gate_id" gets "gate_id", then we know which wire to use as input to another gate
    pub wj_id: BigUint,
    pub is_input_gate: bool,
}
