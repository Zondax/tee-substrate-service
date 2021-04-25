use k256::ecdsa::{recoverable::Signature, signature::DigestSigner, SigningKey};

use crate::util::CSPRNG;

pub struct Keypair {
    secret: SigningKey,
    public: [u8; 33],
}

impl Clone for Keypair {
    fn clone(&self) -> Self {
        let secret = self.secret.to_bytes();
        let secret = SigningKey::from_bytes(&secret).unwrap();

        Self {
            secret,
            public: self.public,
        }
    }
}

impl Keypair {
    pub fn generate<C: CSPRNG>(rng: &mut C) -> Self {
        let mut seed: [u8; 32] = Default::default();
        rng.fill_bytes(&mut seed);

        //not gonna error since length is ok
        let secret = SigningKey::from_bytes(&seed).unwrap();
        let public = secret.verify_key().to_bytes();

        Self { secret, public }
    }

    pub fn public(&self) -> &[u8] {
        &self.public
    }

    pub fn sign(&self, msg: &[u8]) -> [u8; 65] {
        let digest = {
            use blake2::{Blake2s, Digest};
            let mut blake2 = Blake2s::new();
            blake2.update(msg);
            blake2
        };

        let signature: Signature = self.secret.sign_digest(digest);

        {
            let mut array = [0; 65];
            array.copy_from_slice(signature.as_ref());
            array
        }
    }
}
