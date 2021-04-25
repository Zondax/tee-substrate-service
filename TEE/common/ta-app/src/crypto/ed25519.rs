use ed25519_dalek::{SecretKey, SECRET_KEY_LENGTH};

use crate::util::CSPRNG;

pub struct Keypair(ed25519_dalek::Keypair);

impl Clone for Keypair {
    fn clone(&self) -> Self {
        let secret = self.0.secret.as_bytes();
        let secret = SecretKey::from_bytes(&secret[..]).unwrap();

        let keypair = ed25519_dalek::Keypair {
            public: (&secret).into(),
            secret
        };

        Self(keypair)
    }
}

impl Keypair {
    pub fn generate<C: CSPRNG>(rng: &mut C) -> Self {
        let mut seed: [u8; SECRET_KEY_LENGTH] = Default::default();
        rng.fill_bytes(&mut seed);

        //not gonna error since length is ok
        let secret = SecretKey::from_bytes(&seed).unwrap();

        let keypair = ed25519_dalek::Keypair {
            public: (&secret).into(),
            secret,
        };

        Self(keypair)
    }

    pub fn public(&self) -> &[u8] {
        self.0.public.as_ref()
    }

    pub fn sign(&self, msg: &[u8]) -> [u8; 64] {
        use ed25519_dalek::Signer;
        self.0.sign(msg).to_bytes()
    }
}
