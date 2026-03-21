use k256::elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
use k256::{AffinePoint, EncodedPoint, ProjectivePoint, PublicKey, Scalar, SecretKey};
use k256::elliptic_curve::{Field};
use num_bigint::BigUint;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::RngCore;
use crate::crypto_utils;
use serde::{Serialize, Deserialize};

pub struct RealKeyPair {
    public_key : PublicKey,
    secret_key : SecretKey
}

impl RealKeyPair {
    pub fn new() -> Self {
        let mut rng = crypto_utils::gen_rng();
        let keypair = gen_keypair(&mut rng);
        RealKeyPair { public_key: keypair.public_key, secret_key: keypair.secret_key }
    }
    pub fn get_pk(&self) -> &PublicKey {
        &self.public_key
    }
    pub fn get_sk(&self) -> &SecretKey {
        &self.secret_key
    }
}

pub struct ObliviousKeyPair {
    public_key : PublicKey
}

impl ObliviousKeyPair {
    pub fn new() -> Self {
        let mut rng = crypto_utils::gen_rng();
        let keypair = gen_obl_keypair(&mut rng);
        ObliviousKeyPair { public_key: keypair.public_key }
    }
    pub fn get_pk(&self) -> &PublicKey {
        &self.public_key
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CipherText {
    pub payload : BigUint,
    pub ephemeral_key : AffinePoint
}

pub fn gen_keypair(rng : &mut ChaCha20Rng) -> RealKeyPair {
    let sk = SecretKey::random(rng); 
    let sk_as_scalar = sk.to_nonzero_scalar();
    let g = ProjectivePoint::GENERATOR; // Publicly known generator
    // let g_as_affine = g.to_affine();
    let public_point = g * sk_as_scalar.as_ref(); 
    let public_point_as_key = PublicKey::from_affine(public_point.to_affine()).ok().unwrap();
    RealKeyPair { public_key : public_point_as_key, secret_key : sk}
}

pub fn gen_obl_keypair(rng : &mut ChaCha20Rng) -> ObliviousKeyPair {
    // try to land a random point on the curve
    for _i in 0..100 {
        let mut bytes = [0u8; 33]; // 256 bits
        rng.fill_bytes(&mut bytes[1..]); 
        
        bytes[0] = 0x02;// Add SEC1 prefix (0x02 indicates an even Y)
    
        let encoded_point= EncodedPoint::from_bytes(&bytes).unwrap();
        
        let point = ProjectivePoint::from_encoded_point(&encoded_point);
        if point.is_some().into() {
            let pk = PublicKey::from_affine(point.unwrap().to_affine()).ok().unwrap();
            return ObliviousKeyPair{ public_key : pk};
        }    
    }  
    panic!("Could not generate an oblivious public key")
}

pub fn encrypt(rng : &mut ChaCha20Rng, pk : &PublicKey, msg : &BigUint) -> CipherText {
    let k = Scalar::random(rng);
    let ephemeral_key = ProjectivePoint::GENERATOR * k;
    let shared_point: ProjectivePoint = pk.to_projective() * &k;
    let shared_point_as_affine = shared_point.to_affine().to_encoded_point(false);
    let sym_key = crypto_utils::sha256(shared_point_as_affine.as_bytes());
    let payload = msg ^ sym_key; // could also do AES_GCM if we want authenticated encryption, but we are creating a passive secure system, so does not matter
    CipherText { payload: payload, ephemeral_key : ephemeral_key.to_affine() }
}

pub fn decrypt(sk : &SecretKey, ct : &CipherText) -> BigUint {
    let secret_scalar = sk.to_nonzero_scalar();
    let ephemeral_key : ProjectivePoint = ct.ephemeral_key.into();
    let shared_point = ephemeral_key * secret_scalar.as_ref();
    let shared_point_as_bytes = shared_point.to_affine().to_encoded_point(false).to_bytes();
    let sym_key = crypto_utils::sha256(&shared_point_as_bytes);

    &ct.payload ^ sym_key
}
