use k256::{
    ecdsa::{
        recoverable::Signature,
        signature::{DigestSigner, DigestVerifier},
        Error, SigningKey, VerifyingKey,
    },
    EncodedPoint,
};

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

    fn prehash_message(msg: &[u8]) -> blake2::Blake2s {
        use blake2::{Blake2s, Digest};
        let mut blake2 = Blake2s::new();
        blake2.update(msg);
        blake2
    }

    pub fn sign(&self, msg: &[u8]) -> [u8; 65] {
        let digest = Self::prehash_message(msg);

        let signature: Signature = self.secret.sign_digest(digest);

        {
            let mut array = [0; 65];
            array.copy_from_slice(signature.as_ref());
            array
        }
    }
}

#[derive(Debug)]
pub struct PublicKey(VerifyingKey);

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let point = EncodedPoint::from_bytes(bytes).map_err(|_| Error::new())?;

        VerifyingKey::from_encoded_point(&point).map(Self)
    }

    pub fn verify(&self, msg: &[u8], sig: &[u8; 65]) -> bool {
        use core::convert::TryFrom;

        let sig = match Signature::try_from(&sig[..]) {
            Err(_) => return false,
            Ok(sig) => sig,
        };

        let digest = Keypair::prehash_message(msg);

        self.0.verify_digest(digest, &sig).is_ok()
    }
}

impl From<Keypair> for PublicKey {
    fn from(pair: Keypair) -> Self {
        Self(pair.secret.verify_key())
    }
}
