// use num_bigint::BigUint;
// use crate::gates::gates::GateType;
// use crate::wires::wires::Wires;
// use crate::crypto_utils::{gc_kdf_128, generate_label_lsb, generate_label};

// pub struct FreeXORWires {
//     w0: BigUint,
//     w1: BigUint,
//     delta: BigUint,
// }

// impl FreeXORWires {
//     fn delta(&self) -> &BigUint {
//         &self.delta
//     }
// }

// impl Wires for FreeXORWires {
//     fn new(w0: BigUint, w1: BigUint) -> Self {
//         let delta = generate_label_lsb(true); // to ensure point & permute holds
//         Self { w0, w1, delta} 
//     }
//     fn w0(&self) -> &BigUint {
//         &self.w0
//     }

//     fn w1(&self) -> &BigUint {
//         &self.w1
//     }

//     fn generate_input_wire() -> Self {
//         let w0 = generate_label();
//         let w1 = &w0 ^ Self::delta(&self);
//         (w0, w1)
//     }
//     fn generate_output_wire(wi: &Self, wj: &Self, gate: &GateType, gate_id: &BigUint) -> Self {
//         match gate.as_str() {
//             "and"=>generate_and_wires(&self.delta, wi, wj, gate_id),
//             "xor"=>generate_xor_wires(&self.delta, wi, wj, gate_id),
//             _=>panic!("Unknown gate {}", gate),
//         }
//     }
// }

// pub fn generate_and_wires(delta: &BigUint, wi: &(BigUint, BigUint), wj: &(BigUint, BigUint), gate_id: &BigUint) -> (BigUint, BigUint) {
//     let w0c = &wi.0 ^ &wj.0;
//     let w1c = &w0c ^ delta;
//     (w0c, w1c)
// }

// pub fn generate_xor_wires(delta: &BigUint, wi: &(BigUint, BigUint), wj: &(BigUint, BigUint), gate_id: &BigUint) -> (BigUint, BigUint) {
//     let w0c = &wi.0 ^ &wj.0;
//     let w1c = &w0c ^ delta;
//     (w0c, w1c)
// }

// pub fn get_00_wire(wi: &FreeXORWires, wj: &FreeXORWires, gate_id: &BigUint) -> BigUint {
//     for left in [&wi.0, &wi.1] {
//         for right in [&wj.0, &wj.1] {
//             if !left.bit(0) && !right.bit(0) {
//                 return gc_kdf_128(&left, &right, gate_id)
//             }
//         }
//     }
//     panic!("Couldn't find where both wires lsb was 0");
// }