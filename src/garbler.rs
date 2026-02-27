use num_bigint::{BigUint, ToBigUint};

use crate::{circuit_builder::GateBuild, crypto_utils, gates::gates::{Gate, GateType, Gates}, ot::ot::{self, CipherText, PublicKey, PublicParameters}, wires::wires::{Wire, Wires}};

pub struct Garbler<G: Gates<W>, W: Wires> {
    gate_gen: G,
    wire_gen: W,
}

impl<G: Gates<W>, W: Wires> Garbler<G, W> {
    pub fn new(gate: G, wire: W) -> Self {
        Self {
            gate_gen: gate,
            wire_gen: wire,
        }
    }
    pub fn create_circuit(&self, circuit_build : &Vec<GateBuild>, garblers_input_choices : &Vec<u8>,  evaluators_input_choices : &Vec<[ (&PublicKey, &PublicParameters) ; 2]>) -> (Vec<Gate>, Vec<BigUint>, Vec<(CipherText, CipherText)>, [(BigUint, u8); 2])  {
        let mut circuit : Vec<Gate> = vec![];
        let mut wi_inputs : Vec<BigUint> = vec![];
        let mut wj_inputs : Vec<(CipherText, CipherText)> = vec![];
        let mut gate_index = 0;
        let mut conversion_data: [(BigUint, u8); 2] = [
        (BigUint::from(0u32), 0), (BigUint::from(0u32), 0)
        ];

        for gate in circuit_build {
            let wi = self.wire_gen.generate_input_wire();
            let wj = self.wire_gen.generate_input_wire();
            let wj_encrypted;
            
            let gate_is_input_layer = gate.wo().output_layer() == &1.to_biguint().unwrap();
            if gate_is_input_layer { // Encrypt with received publickeys from OT. The real and the oblivious
                let wj_0_ct  = ot::encrypt(&evaluators_input_choices[gate_index][0].1, &evaluators_input_choices[gate_index][0].0, wj.w0());
                let wj_1_ct  = ot::encrypt(&evaluators_input_choices[gate_index][1].1, &evaluators_input_choices[gate_index][1].0, wj.w1());
                
                wj_encrypted = (wj_0_ct, wj_1_ct);
                let garbler_input_choice = garblers_input_choices[gate_index];
                let selected_wire = match garbler_input_choice {
                    0 => wi.w0().clone(), 1 => wi.w1().clone(), _ => panic!("Invalid bit value: must be 0 or 1"),
                };
                wi_inputs.push(selected_wire);
                wj_inputs.push(wj_encrypted);
            }
            
            let new_gate = self.gate_gen.generate_gate(gate.gate_type().clone(), wi.clone(), wj.clone(), 1.to_biguint().unwrap());
            if gate == &circuit_build[circuit_build.len() - 1] {
                conversion_data = [
                    (new_gate.wo.w0().clone(), 0), (new_gate.wo.w1().clone(), 1)
                    ];
                } 
                
            circuit.push(new_gate);
            gate_index+= 1;
        }
        (circuit, wi_inputs, wj_inputs, conversion_data)
    }
}
