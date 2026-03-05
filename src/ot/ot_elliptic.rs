use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::{NonZeroScalar, ProjectivePoint, Scalar, SecretKey};
use k256::elliptic_curve::{Field};
use num_bigint::BigUint;
use rand_chacha::ChaCha20Rng;
use crate::crypto_utils;

pub struct KeyPair {
    public_key : ProjectivePoint,
    secret_key : NonZeroScalar
}

impl KeyPair {
    pub fn new() -> Self {
        let mut rng = crypto_utils::gen_rng();
        let keypair = gen_keypair(&mut rng);
        KeyPair { public_key: keypair.public_key, secret_key: keypair.secret_key }
    }
    pub fn get_pk(&self) -> &ProjectivePoint {
        &self.public_key
    }
    pub fn get_sk(&self) -> &NonZeroScalar {
        &self.secret_key
    }
}

pub fn gen_keypair(rng : &mut ChaCha20Rng) -> KeyPair {
    let secret_point = SecretKey::random(rng).to_nonzero_scalar(); // secret key should never be zero
    let g = ProjectivePoint::GENERATOR; // Publicly known generator
    let public_point = g * secret_point.as_ref();
    KeyPair { public_key : public_point, secret_key : secret_point }
}

pub fn encrypt(rng : &mut ChaCha20Rng, msg : &BigUint, pk : &ProjectivePoint) -> (BigUint, ProjectivePoint) {
    let k = Scalar::random(rng);
    let c1 = ProjectivePoint::GENERATOR * k;
    let shared_point = pk * &k;
    let shared_point_as_affine = shared_point.to_affine().to_encoded_point(false);
    let sym_key = crypto_utils::sha256(shared_point_as_affine.as_bytes());
    let ct = msg ^ sym_key; // could also do AES_GCM if we want authenticated encryption, but we are creating a passive secure system, so does not matter
    (ct, c1)
}

pub fn decrypt(ct : BigUint, c1 : ProjectivePoint, sk : &NonZeroScalar) -> BigUint {
    let shared_point = c1 * sk.as_ref();
    let shared_point_as_bytes = shared_point.to_affine().to_encoded_point(false).to_bytes();
    let sym_key = crypto_utils::sha256(&shared_point_as_bytes);

    ct ^ sym_key
}
