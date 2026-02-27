use std::ops::Mul;
use num_bigint::{BigUint, ToBigUint};
use glass_pumpkin::safe_prime;
use rand::{thread_rng, Rng};


// Global parameters for the group used in OT
#[derive(Debug)]
pub struct PublicParameters {
    p: BigUint, // Public modulus
    q: BigUint, // Prime subgroup order
    g: BigUint, // Group generator
}

impl PublicParameters {
    pub fn new() -> Self {
        let p: BigUint = Self::generate_safe_prime();
        let q = (&p-1.to_biguint().unwrap())/2.to_biguint().unwrap();
        let g = Self::generate_generator(&p, &q);
        PublicParameters {
            p,
            q,
            g,
        }
    }

    fn generate_safe_prime() -> BigUint {
        safe_prime::new(200).unwrap()
    }


    fn generate_generator(p: &BigUint, q: &BigUint) -> BigUint {
        let mut g_candidate = 10.to_biguint().unwrap();
        loop {
            if g_candidate.modpow(q, p) == 1.to_biguint().unwrap() {
                return g_candidate;
            }
        g_candidate += 1.to_biguint().unwrap();
        }
    }

    pub fn get_p(&self) -> &BigUint {
        &self.p
    }

    pub fn get_q(&self) -> &BigUint {
        &self.q
    }

    pub fn get_g(&self) -> &BigUint {
        &self.g
    }
}

#[derive(Clone, Debug)]
pub struct SecretKey {
    alpha: BigUint,
}

impl SecretKey {
    pub fn get_alpha(&self) -> &BigUint {
        &self.alpha
    }
}

// Can either be a real or a fake public key.
// Real: h is a real discrete log
// Fake: h is a random element in the prime order subgroup of size q of the Z_p^*
#[derive(Clone, Debug)]
pub struct PublicKey {
    g: BigUint, // Group generator
    h: BigUint,
}

impl PublicKey {
    pub fn get_g(&self) -> &BigUint {
        &self.g
    }
    pub fn get_h(&self) -> &BigUint {
        &self.h
    }
}

// Regular key generation
pub struct RealKeyPair {
    secret_key: SecretKey,
    public_key: PublicKey,
}

impl RealKeyPair {
    pub fn new(pp: &PublicParameters) -> Self {
        let sk = RealKeyPair::secret_key_generator(&pp.q);
        let pk = RealKeyPair::public_key_generator(&pp, &sk);
        RealKeyPair {
            secret_key: sk,
            public_key: pk,
        }
    }

    // Generates a random number \alpha \in Z_q.
    // if q=p-1/2 is at most 2000 bits, then we pick a random 2000 bit number \alpha, and make
    // sure that 0 <= \alpha <= q-1 and if not, we try again.
    fn secret_key_generator(q: &BigUint) -> SecretKey {
        let mut bytes = [0u8, 125];
        loop {
            thread_rng().fill(&mut bytes);
            let alpha = BigUint::from_bytes_be(&bytes);
            if alpha < *q {
                let sk = SecretKey {
                    alpha
                };
                return sk;
            }
        }
    }

    fn public_key_generator(pp: &PublicParameters, sk: &SecretKey) -> PublicKey {
        PublicKey {
            g: pp.g.clone(),
            h: pp.g.modpow(&sk.alpha, &pp.p),
        }
    }

    pub fn get_public_key(&self) -> PublicKey {
        self.public_key.clone()
    }
    pub fn get_secret_key(&self) -> SecretKey {
        self.secret_key.clone()
    }
}


    // Oblivious key generation
pub struct ObliviousKeyPair {
    public_key: PublicKey,
}

impl ObliviousKeyPair {
    pub fn new(pp: &PublicParameters) -> Self {
        ObliviousKeyPair {
            public_key : Self::public_key_generator(&pp)
        }
    }
    pub fn get_public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    fn public_key_generator(pp: &PublicParameters) -> PublicKey {
        // h is set as random element in Z_p after which it is converted to the subgroup of Z_p of order q
        // by squaring it mod p
        let mut bytes = [0, 250];
        let mut b: BigUint;
        loop {
            thread_rng().fill(&mut bytes);
            b = BigUint::from_bytes_be(&bytes);
            if b < pp.p {
                break;
            }
        }
        let h = b.modpow(&2.to_biguint().unwrap(), &pp.p);
        PublicKey {
            g: pp.g.clone(),
            h,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CipherText {
    pub c_1: BigUint,
    pub c_2: BigUint,
}

impl CipherText {
    pub fn get_c_1(&self) -> &BigUint {
        &self.c_1
    }
    pub fn get_c_2(&self) -> &BigUint {
        &self.c_2
    }
}

pub fn encrypt(pp: &PublicParameters, public_key: &PublicKey, plaintext: &BigUint) -> CipherText {
    // source randomness for encryption
    let r = generate_r(pp.get_q());
    let c_1 = public_key.get_g().modpow(&r, &pp.p);
    // this need to calculate m*h^r mod p but u can only exponentiate in combination with modulus.
    // so first we calculate h^r mod p, and then do m*(h^r) mod p
    let c_2 = (plaintext*public_key.get_h().modpow(&r, &pp.p)).modpow(&1.to_biguint().unwrap(), &pp.p);
    CipherText {
        c_1,
        c_2,
    }
}

fn generate_r(q: &BigUint) -> BigUint {
    let mut bytes = [0u8, 125];
    loop {
        thread_rng().fill(&mut bytes);
        let r = BigUint::from_bytes_be(&bytes);
        if r < *q {
            return r;
        }
    }
}

pub fn decrypt(pp: &PublicParameters, secret_key: SecretKey, ciphertext: CipherText) -> BigUint {
    println!("Secretkey to decrypt: {:?}", secret_key);
    let c_1_pow_alpha = ciphertext.get_c_1().modpow(secret_key.get_alpha(), &pp.p);
    let c_1_pow_alpha_inv = c_1_pow_alpha.modinv(&pp.p).unwrap();
    (&ciphertext.c_2 * &c_1_pow_alpha_inv) % &pp.p
}