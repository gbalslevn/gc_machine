use num_bigint::{BigUint};
pub trait Gates {
    fn get_garbled_gate(tt : &[(BigUint, BigUint, BigUint); 4], gate_id: &BigUint) -> Vec<BigUint>;
    fn get_xor_tt(w0i: &BigUint, w1i: &BigUint, w0j: &BigUint, w1j: &BigUint, w0c: &BigUint, w1c: &BigUint) -> [(BigUint, BigUint, BigUint); 4] {
        [(w0i.clone(), w0j.clone(), w0c.clone()), (w0i.clone(), w1j.clone(), w1c.clone()), (w1i.clone(), w0j.clone(), w1c.clone()), (w1i.clone(), w1j.clone(), w0c.clone())] // should avoid using clone if wanting performancee
    }
    fn get_and_tt(w0i: &BigUint, w1i: &BigUint, w0j: &BigUint, w1j: &BigUint, w0c: &BigUint, w1c: &BigUint) -> [(BigUint, BigUint, BigUint); 4] {
        [(w0i.clone(), w0j.clone(), w0c.clone()), (w0i.clone(), w1j.clone(), w0c.clone()), (w1i.clone(), w0j.clone(), w0c.clone()), (w1i.clone(), w1j.clone(), w1c.clone())]
    }
}