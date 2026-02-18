use num_bigint::{BigUint, ToBigUint};
use glass_pumpkin::safe_prime;
use rand::{thread_rng, Rng};


// Global parameters for the group used in OT
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

    pub fn get_p(self) -> BigUint {
        self.p.clone()
    }

    fn generate_safe_prime() -> BigUint {
        safe_prime::new(128).unwrap()
    }


    fn generate_generator(p: &BigUint, q: &BigUint) -> BigUint {
        println!("start finding generator");
        let mut g_candidate = 10.to_biguint().unwrap();
        loop {
            println!("Current candidate: {} \nAnd the exponenet is: {} \nAnd the mod is: {} \nAnd the result is {}", g_candidate, q, p, g_candidate.modpow(q, p));
            if g_candidate.modpow(q, p) == 1.to_biguint().unwrap() {
                println!("finished finding generator");
                return g_candidate;
            }
        g_candidate += 1.to_biguint().unwrap();
        }
    }
}

#[derive(Clone, Debug)]
struct SecretKey {
    alpha: BigUint, // secret exponent
}

    // Can either be a real or a fake public key.
    // Real: h is a real discrete log
    // Fake: h is a random element in the subgroup of size q of the Z_p^*
    struct PublicKey {
        g: BigUint, // Group generator
        h: BigUint,
    }

    // Regular key generation
    struct RealKeyPair {
        secret_key: SecretKey,
        public_key: PublicKey,
    }

    impl RealKeyPair {
        fn new(pp: PublicParameters) -> Self {
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
            let mut bytes = [0u8, 250];
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
    }






    // Oblivious key generation
    struct ObliviousKeyPair {
        public_key: PublicKey,
    }