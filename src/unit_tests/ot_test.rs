use glass_pumpkin::safe_prime;
use num_bigint::{BigUint, ToBigUint};
use crate::{crypto_utils, ot::{eg_elliptic, eg_finite_field::{self, PublicParameters, RealKeyPair}}, wires::{point_and_permute_wire_gen::PointAndPermuteWireGen, wire_gen::WireGen}};

#[cfg(test)]

fn setup() -> eg_finite_field::PublicParameters {
    eg_finite_field::PublicParameters::new()
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
    let oblivious_key = eg_finite_field::ObliviousKeyPair::new(&pp);
    let h = oblivious_key.get_public_key().get_h().clone();
    assert_eq!(h.modpow(pp.get_q(), &pp.get_p()), 1.to_biguint().unwrap());
}

#[test]
fn real_pk_should_decrypt_correctly() {
    let pp = PublicParameters::new();
    let real_keypair = RealKeyPair::new(&pp);
    let mut wire_gen = PointAndPermuteWireGen::new();
    let plaintext  = wire_gen.generate_input_wire().w0().clone();
    let cipher_text = eg_finite_field::encrypt( &pp, &real_keypair.get_public_key(), &plaintext);
    let decrypted_ciphertext = eg_finite_field::decrypt(&pp, &real_keypair.get_secret_key(), &cipher_text);
    assert!(plaintext == decrypted_ciphertext)
}

#[test]
fn elliptic_can_decrypt() {
    // secp256k1 is defined by the equation y^2=x^3+7
    // Projective and affine are two ways to represent points on a curve. 
    // Affine is with coordinates (x, y), where every point (x,y) which satisfes the EC equation, is a point on the curve. 
    // A projective point is represented as (X,Y,Z) and is more fuzzy. The same point on the curve can be represented as many different (X,Y,Z). It is good to use for math. Affine is better for communication 

    // We get 128 bit security from the a 256 bit size (as k256 uses) of the elliptic curve group as Pollard's rho algorithm allows finding discrete log in sqrt(n) where n is 2^bitsize. This results in approx half security amount for the bitsize.

    let mut rng = crypto_utils::gen_rng();
    
    // 1. Receiver gen a keypair
    let keypair = eg_elliptic::RealKeyPair::new();

    // 2. Encryption (Sender)
    let message = BigUint::from(123456789u64); 
    let ct = eg_elliptic::encrypt(&mut rng, keypair.get_pk(), &message);

    // 4. Decryption (Receiver)
    let pt = eg_elliptic::decrypt(&keypair.get_sk(), &ct);
    
    assert_eq!(message, pt);
}