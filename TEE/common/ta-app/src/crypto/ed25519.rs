use ed25519_dalek::{SecretKey, Signature, SignatureError, SECRET_KEY_LENGTH};

use crate::util::CSPRNG;

pub struct Keypair(ed25519_dalek::Keypair);

impl Clone for Keypair {
    fn clone(&self) -> Self {
        let secret = self.0.secret.as_bytes();
        let secret = SecretKey::from_bytes(&secret[..]).unwrap();

        secret.into()
    }
}

impl Keypair {
    pub fn generate<C: CSPRNG>(rng: &mut C) -> Self {
        let mut seed: [u8; SECRET_KEY_LENGTH] = Default::default();
        rng.fill_bytes(&mut seed);

        //not gonna error since length is ok
        let secret = SecretKey::from_bytes(&seed).unwrap();

        secret.into()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let secret = SecretKey::from_bytes(bytes).ok()?;

        Some(secret.into())
    }

    pub fn public(&self) -> &[u8] {
        self.0.public.as_ref()
    }

    pub fn sign(&self, msg: &[u8]) -> [u8; 64] {
        use ed25519_dalek::Signer;
        self.0.sign(msg).to_bytes()
    }
}

#[derive(Debug)]
pub struct PublicKey(ed25519_dalek::PublicKey);

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SignatureError> {
        ed25519_dalek::PublicKey::from_bytes(bytes).map(Self)
    }

    pub fn verify(&self, msg: &[u8], sig: &[u8; 64]) -> bool {
        use ed25519_dalek::Verifier;

        let sig = Signature::from(*sig);
        self.0.verify(msg, &sig).is_ok()
    }
}

impl From<&Keypair> for PublicKey {
    fn from(pair: &Keypair) -> Self {
        Self(pair.0.public)
    }
}

impl From<SecretKey> for Keypair {
    fn from(secret: SecretKey) -> Self {
        Self(ed25519_dalek::Keypair {
            public: (&secret).into(),
            secret,
        })
    }
}
