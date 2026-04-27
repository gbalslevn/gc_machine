use std::collections::{HashMap, VecDeque};

use crate::circuit_builder::{BuildType, StackBuild};
use crate::crypto_utils::{gc_kdf, gc_kdf_mux};
use crate::{circuit_builder::CircuitBuild,gates::gate_gen::GateGen,ot::eg_elliptic::{self, CipherText},wires::wire_gen::{Wire, WireGen},
};
use k256::PublicKey;
use num_bigint::{BigUint, ToBigUint};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use crate::circuit_builder::{SubcircuitBuild};
use crate::gates::half_gates_gate_gen::HalfGatesGateGen;

#[derive(Serialize, Deserialize, Debug)]
pub struct Circuit {
    pub gates: Vec<Vec<BigUint>>,
    pub constant_wires: Vec<BigUint>,
    pub garbler_input: HashMap<BigUint, BigUint>,
    pub evaluator_input: HashMap<BigUint, (CipherText, CipherText)>,
    pub output_conversion: Vec<[(BigUint, u8); 2]>,
    pub stacks: HashMap<BigUint, Stack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    pub demux: Vec<BigUint>,
    pub stacked_m: Vec<Vec<BigUint>>,
    pub mux: Vec<BigUint>,
}

impl Circuit {
    pub fn new(
        gates: Vec<Vec<BigUint>>,
        constant_wires: Vec<BigUint>,
        garbler_input: HashMap<BigUint, BigUint>,
        evaluator_input: HashMap<BigUint, (CipherText, CipherText)>,
        output_conversion: Vec<[(BigUint, u8); 2]>,
        stacks: HashMap<BigUint, Stack>
    ) -> Self {
        Circuit {
            gates,
            constant_wires,
            garbler_input,
            evaluator_input,
            output_conversion,
            stacks,
        }
    }
}

pub struct Garbler<G: GateGen> {
    pub gate_gen: G,
}

impl<G: GateGen> Garbler<G> {
    pub fn new(gate_gen: G) -> Self {
        Self { gate_gen: gate_gen }
    }

    pub fn create_circuit(
        &mut self,
        circuit_build: &CircuitBuild,
        garblers_input_choices: &VecDeque<u8>,
        evaluators_input_choices: &VecDeque<[PublicKey; 2]>,
    ) -> Circuit {
        if garblers_input_choices.len() != evaluators_input_choices.len() {
            panic!("Garbler and evaluator input length must be equal")
        }
        let mut garbled_gates: Vec<Vec<BigUint>> = Vec::new();
        let mut stacks : HashMap<BigUint, Stack> = HashMap::new();
        let mut known_wires: HashMap<BigUint, Wire> = HashMap::new();
        let mut wi;
        let mut wj;
        let mut output_conversion: Vec<[(BigUint, u8); 2]> = Vec::new();
        let builds = circuit_build.get_builds();

        // insert constants for true and false wire into known_wires, to enable eg. NOT gates
        let constant_wires = self.insert_constant_wires(&mut known_wires);
        // insert garbler and evaluators input wires
        let (garbler_inputs, evaluator_inputs) = self.insert_input_wires(
            &mut known_wires,
            &circuit_build,
            &mut garblers_input_choices.clone(),
            &mut evaluators_input_choices.clone(),
        );

        for build in builds {
            match build.get_type() {
                BuildType::Gate => {
                    let gate = build.unwrap_to_gate();
                    // Generate gates with the inputs
                    wi = known_wires.get(&gate.wi().wire_id()).unwrap().clone();
                    wj = known_wires.get(&gate.wj().wire_id()).unwrap().clone();

                    let new_gate = self.gate_gen.generate_gate(
                        gate.gate_type().clone(),
                        wi.clone(),
                        wj.clone(),
                    );

                    let output_wire_id = gate.wo().wire_id();
                    known_wires.insert(output_wire_id.clone(), new_gate.wo.clone());
                    let table = new_gate.to_table();

                    if circuit_build.output_wires.contains(gate.wo()) {
                        output_conversion
                            .push([(new_gate.wo.w0().clone(), 0), (new_gate.wo.w1().clone(), 1)]);
                    }

                    // Store the ciphertexts for the gate
                    garbled_gates.push(table);
                }
                BuildType::Stack => {
                    let stack = build.unwrap_to_stack();
                    let seed = known_wires.get(&stack.conditional.wire_id()).unwrap().clone();
                     // insert all input wires
                    let mut input_wires = vec![];
                    for input_wire in &stack.input_wires {
                        input_wires.push(known_wires.get(input_wire.wire_id()).unwrap());
                    }
                    // generate material for both branches
                    let (c0_input_wires, c0, c0_output_wires ) = self.generate_subcircuit(seed.w0(), &stack.if_circuit);
                    let (c0_garbage_input_wires, __c0_garbage,c0_garbage_output_wires ) = self.generate_subcircuit(seed.w1(), &stack.if_circuit);
                    let (c1_input_wires,c1,c1_output_wires ) = self.generate_subcircuit(seed.w1(), &stack.else_circuit);
                    let (c1_garbage_input_wires, __c1_garbage,c1_garbage_output_wires ) = self.generate_subcircuit(seed.w0(), &stack.else_circuit);
                    for i in 0..c0_input_wires.len() { // We can use both input and output length of any of the wires as they have equal elements
                        let demux = self.generate_demux(input_wires[i], &seed, &c0_input_wires[i], &c1_input_wires[i], &c0_garbage_input_wires[i], &c1_garbage_input_wires[i]);
                        let stacked_m = self.stack_material(&c0, &c1);
                        let mux = self.generate_mux(&seed, &c0_output_wires[i], &c1_output_wires[i], &c0_garbage_output_wires[i], &c1_garbage_output_wires[i]);
                        stacks.insert(stack.id.to_biguint().unwrap(), Stack {demux, stacked_m, mux});
                    }
                }
            }
        }
        Circuit::new(
            garbled_gates,
            constant_wires,
            garbler_inputs,
            evaluator_inputs,
            output_conversion,
            stacks
        )
    }

    pub fn stack_material(&mut self, c0: &Vec<Vec<BigUint>>, c1: &Vec<Vec<BigUint>>) -> Vec<Vec<BigUint>> {
        // This function makes an XOR between each garbled entry in the stacked material m_cond and the generated subcircuit
        let stacked_material: Vec<Vec<BigUint>> = c0
            .iter()
            .zip(c1.iter())
            .map(|(c0_row, c1_row)| {
                c0_row
                    .iter()
                    .zip(c1_row.iter())
                    .map(|(c0_val, c1_val)| c0_val ^ c1_val)
                    .collect()
            })
            .collect();
        stacked_material
    }

    pub fn create_circuit_input(&self, input: &BigUint, required_bits: u64) -> VecDeque<u8> {
        let mut list = VecDeque::new();
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

    fn generate_stack(&self, stack: StackBuild) {

    }

    fn generate_demux(
        &self,
        input_wire: &Wire,
        conditional: &Wire,
        if_wire: &Wire,
        else_wire: &Wire,
        garbage_if_wire: &Wire,
        garbage_else_wire: &Wire,
    ) -> Vec<BigUint> {
        let mut table = vec![BigUint::from(0u8); 4];

        let entry_0 = gc_kdf(input_wire.w0(), conditional.w0(), &1.to_biguint().unwrap())
            ^ ((if_wire.w0() << 128) | garbage_else_wire.w0());
        let entry_1 = gc_kdf(input_wire.w1(), conditional.w0(), &1.to_biguint().unwrap())
            ^ ((if_wire.w1() << 128) | garbage_else_wire.w1());
        let entry_2 = gc_kdf(input_wire.w0(), conditional.w1(), &1.to_biguint().unwrap())
            ^ ((garbage_if_wire.w0() << 128) | else_wire.w0());
        let entry_3 = gc_kdf(input_wire.w1(), conditional.w1(), &1.to_biguint().unwrap())
            ^ ((garbage_if_wire.w1() << 128) | else_wire.w1());
        let entry_0_pos = get_position(input_wire.w0(), conditional.w0());
        let entry_1_pos = get_position(input_wire.w1(), conditional.w0());
        let entry_2_pos = get_position(input_wire.w0(), conditional.w1());
        let entry_3_pos = get_position(input_wire.w1(), conditional.w1());
        table[entry_0_pos] = entry_0;
        table[entry_1_pos] = entry_1;
        table[entry_2_pos] = entry_2;
        table[entry_3_pos] = entry_3;
        table
    }

    fn generate_mux(
        &self,
        conditional: &Wire,
        if_wire: &Wire,
        else_wire: &Wire,
        garbage_if_wire: &Wire,
        garbage_else_wire: &Wire,
    ) -> Vec<BigUint> {
        let mut table = vec![BigUint::from(0u8); 4];
        let entry_0 = gc_kdf_mux(
            conditional.w0(),
            if_wire.w0(),
            garbage_else_wire.w0(),
            &1.to_biguint().unwrap(),
        ) ^ ((if_wire.w0() << 128) | garbage_else_wire.w0());
        let entry_1 = gc_kdf_mux(
            conditional.w0(),
            if_wire.w1(),
            garbage_else_wire.w0(),
            &1.to_biguint().unwrap(),
        ) ^ ((if_wire.w1() << 128) | garbage_else_wire.w1());
        let entry_2 = gc_kdf_mux(
            conditional.w1(),
            garbage_if_wire.w0(),
            else_wire.w1(),
            &1.to_biguint().unwrap(),
        ) ^ ((garbage_if_wire.w0() << 128) | else_wire.w0());
        let entry_3 = gc_kdf_mux(
            conditional.w1(),
            garbage_if_wire.w1(),
            else_wire.w1(),
            &1.to_biguint().unwrap(),
        ) ^ ((garbage_if_wire.w1() << 128) | else_wire.w1());
        let entry_0_pos = get_position(if_wire.w0(), garbage_else_wire.w0());
        let entry_1_pos = get_position(if_wire.w1(), garbage_else_wire.w1());
        let entry_2_pos = get_position(garbage_if_wire.w1(), else_wire.w0());
        let entry_3_pos = get_position(garbage_if_wire.w0(), else_wire.w1());
        table[entry_0_pos] = entry_0;
        table[entry_1_pos] = entry_1;
        table[entry_2_pos] = entry_2;
        table[entry_3_pos] = entry_3;
        table
    }

    fn generate_subcircuit(&self, seed: &BigUint, subcircuit_build: &SubcircuitBuild) -> (Vec<Wire>, Vec<Vec<BigUint>>, Vec<Wire>) { // make a subcircuit datatype
        let mut gate_gen = HalfGatesGateGen::new_with_seed(seed);
        let subcircuit_input_wires = &subcircuit_build.input_wires;
        let mut known_wires = HashMap::new();
        
        // generate all input wires for the subcircuit with the set seed
        for input_wire in subcircuit_input_wires {
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
        let mut input_wires = vec![];
        let mut output_wires = vec![];
        for wire_build in &subcircuit_build.input_wires {
            let input_wire = known_wires.get(wire_build.wire_id()).unwrap();
            input_wires.push(input_wire.clone());
        }
        for wire_build in &subcircuit_build.output_wires {
            let output_wire = known_wires.get(wire_build.wire_id()).unwrap();
            output_wires.push(output_wire.clone());
        }
        (input_wires, subcircuit, output_wires)
    }

    // Inserts the garbler and evaluators input wires
    fn insert_input_wires(
        &mut self,
        known_wires: &mut HashMap<BigUint, Wire>,
        build: &CircuitBuild,
        garblers_input_choices: &mut VecDeque<u8>,
        evaluators_input_choices: &mut VecDeque<[PublicKey; 2]>,
    ) -> (
        HashMap<BigUint, BigUint>,
        HashMap<BigUint, (CipherText, CipherText)>,
    ) {
        let mut rng = self.gate_gen.get_wire_gen().get_rng().clone();
        let garbler_wires = &build.garbler_wires;
        let evaluator_wires = &build.evaluator_wires;

        // Insert garbler input and save the garbler input choice in a map of labels
        let mut garbler_inputs = HashMap::new();
        for wirebuild in garbler_wires {
            let wire = self.gate_gen.get_wire_gen().generate_input_wire();
            let garbler_input_choice = garblers_input_choices.pop_front().unwrap();
            let selected_wirelabel = match garbler_input_choice {
                0 => wire.w0(),
                1 => wire.w1(),
                _ => panic!("Invalid bit value: must be 0 or 1"),
            };
            garbler_inputs.insert(wirebuild.wire_id().clone(), selected_wirelabel.clone());
            known_wires.insert(wirebuild.wire_id().clone(), wire.clone());
        }

        // Insert evaluator input wires and save each label for the wire as a ciphertext where the evaluator can decrypt from OT
        let mut evaluator_inputs = HashMap::new();
        for wirebuild in evaluator_wires {
            let wire = self.gate_gen.get_wire_gen().generate_input_wire();
            let wire_encrypted = self.gen_encrypted_wire(
                &wire,
                &evaluators_input_choices.pop_front().unwrap(),
                &mut rng,
            );
            evaluator_inputs.insert(wirebuild.wire_id().clone(), wire_encrypted.clone());
            known_wires.insert(wirebuild.wire_id().clone(), wire.clone());
        }

        (garbler_inputs, evaluator_inputs)
    }

    fn insert_constant_wires(&mut self, known_wires: &mut HashMap<BigUint, Wire>) -> Vec<BigUint> {
        let mut constant_wires = vec![];
        let true_constant = self.gate_gen.get_wire_gen().generate_input_wire();
        let false_constant = self.gate_gen.get_wire_gen().generate_input_wire();
        known_wires.insert(0.to_biguint().unwrap(), false_constant.clone());
        known_wires.insert(1.to_biguint().unwrap(), true_constant.clone());
        constant_wires.insert(0, false_constant.w0().clone());
        constant_wires.insert(1, true_constant.w1().clone());
        constant_wires
    }

    // Encrypts a wire for the evaluator as a part of OT
    fn gen_encrypted_wire(
        &self,
        wire: &Wire,
        input_choice: &[PublicKey; 2],
        rng: &mut ChaCha20Rng,
    ) -> (CipherText, CipherText) {
        let pk_0 = input_choice[0];
        let wj_0_ct = eg_elliptic::encrypt(rng, &pk_0, wire.w0());
        let pk_1 = input_choice[1];
        let wj_1_ct = eg_elliptic::encrypt(rng, &pk_1, wire.w1());

        let wj_encrypted = (wj_0_ct, wj_1_ct);
        wj_encrypted
    }
}

fn get_position(wi: &BigUint, wj: &BigUint) -> usize {
    let l = wi.bit(0) as usize;
    let r = wj.bit(0) as usize;
    l * 2 + r
}
