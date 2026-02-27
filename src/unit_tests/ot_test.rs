use glass_pumpkin::safe_prime;
use num_bigint::{BigUint, ToBigUint};
use crate::{ot::ot::{self, CipherText}, wires::{original_wires::OriginalWires, point_and_permute_wires::PointAndPermuteWires, wires::Wires}};

#[cfg(test)]

fn setup() -> ot::PublicParameters {
    ot::PublicParameters::new()
}

#[test]
fn p_should_be_safe_prime() {
    let pp = setup();
    let p = pp.get_p();
    assert!(safe_prime::check(&p));
}

#[test]
fn p_should_be_1000_bits() {
    let pp = setup();
    assert_eq!(pp.get_p().bits(), 200);
}

// q should be exactly equal to q=(p-1)/2
#[test]
fn q_should_divide_p_minus() {
    let pp = setup();
    assert_eq!(pp.get_q()*2.to_biguint().unwrap()+1.to_biguint().unwrap(), *pp.get_p());
}

#[test]
fn g_should_be_generator_of_order_q_subgroup_of_p() {
    let pp = setup();
    assert_eq!(pp.get_g().modpow(&pp.get_q(),&pp.get_p()), 1.to_biguint().unwrap());
}

#[test]
fn oblivious_key_element_h_should_be_in_multiplicative_subgroup() {
    let pp = setup();
    let oblivious_key = ot::ObliviousKeyPair::new(&pp);
    let h = oblivious_key.get_public_key().get_h().clone();
    assert_eq!(h.modpow(pp.get_q(), &pp.get_p()), 1.to_biguint().unwrap());
}

#[test]
fn real_pk_should_decrypt_correctly() {
    let pp = ot::PublicParameters::new();
    let real_keypair = ot::RealKeyPair::new(&pp);
    let wire_gen = PointAndPermuteWires::new();
    let plaintext  = wire_gen.generate_input_wire().w0().clone();
    println!("Pt is: {}", plaintext);
    let cipher_text = ot::encrypt(&pp, &real_keypair.get_public_key(), &plaintext);
    println!("Ct is: {:?}", cipher_text);
    let decrypted_ciphertext = ot::decrypt(&pp, real_keypair.get_secret_key(), cipher_text);
    println!("dc_ct is: {:?}", decrypted_ciphertext);
    assert!(plaintext == decrypted_ciphertext)
}