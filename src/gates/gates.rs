use num_bigint::{BigUint};

pub trait Gates {
    fn get_garbled_gate(tt : &[(BigUint, BigUint, BigUint); 4], gate_id: &BigUint) -> Vec<BigUint>;
    
    fn get_tt(wi: &(BigUint, BigUint), wj: &(BigUint, BigUint), wo: &(BigUint, BigUint), gate: String) -> [(BigUint, BigUint, BigUint); 4] {
        match gate.as_str() {
            "and"=>Self::get_and_tt(wi, wj, wo),
            "xor"=>Self::get_xor_tt(wi, wj, wo),
            _=>panic!("Unknown gate {}", gate),
        }
    }
    fn get_xor_tt(wi: &(BigUint, BigUint), wj: &(BigUint, BigUint), wo: &(BigUint, BigUint)) -> [(BigUint, BigUint, BigUint); 4] {
        [(wi.0.clone(), wj.0.clone(), wo.0.clone()), (wi.0.clone(), wj.1.clone(), wo.1.clone()), (wi.1.clone(), wj.0.clone(), wo.1.clone()), (wi.1.clone(), wj.1.clone(), wo.0.clone())] // should avoid using clone if wanting performancee
    }
    fn get_and_tt(wi: &(BigUint, BigUint), wj: &(BigUint, BigUint), wo: &(BigUint, BigUint)) -> [(BigUint, BigUint, BigUint); 4] {
        [(wi.0.clone(), wj.0.clone(), wo.0.clone()), (wi.0.clone(), wj.1.clone(), wo.0.clone()), (wi.1.clone(), wj.0.clone(), wo.0.clone()), (wi.1.clone(), wj.1.clone(), wo.1.clone())]
    }
}