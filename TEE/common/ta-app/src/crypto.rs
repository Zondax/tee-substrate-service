use super::*;

use crate::util::CSPRNG;
use optee_common::CryptoAlgo;

mod ecdsa;
mod ed25519;
mod sr25519;
pub use sr25519::VRFData;

///Handles all the diffrent supported curves with a unified interface
#[derive(Clone)]
pub enum Keypair {
    Sr25519(sr25519::Keypair),
    Ed25519(ed25519::Keypair),
    Ecdsa(ecdsa::Keypair),
}

impl Keypair {
    pub fn generate_new<C: CSPRNG>(rng: &mut C, algo: CryptoAlgo) -> Self {
        match algo {
            CryptoAlgo::Sr25519 => Self::Sr25519(sr25519::Keypair::generate(rng)),
            CryptoAlgo::Ed25519 => Self::Ed25519(ed25519::Keypair::generate(rng)),
            CryptoAlgo::Ecdsa => Self::Ecdsa(ecdsa::Keypair::generate(rng)),
        }
    }

    pub fn public(&self) -> &[u8] {
        match self {
            Self::Sr25519(kp) => kp.public(),
            Self::Ed25519(kp) => kp.public(),
            Self::Ecdsa(kp) => kp.public(),
        }
    }

    pub fn sign<C: CSPRNG>(&self, rng: &mut C, msg: &[u8]) -> Vec<u8> {
        match self {
            Self::Sr25519(kp) => kp.sign(rng, msg).to_vec(),
            Self::Ed25519(kp) => kp.sign(msg).to_vec(),
            Self::Ecdsa(kp) => kp.sign(msg).to_vec(),
        }
    }

    pub fn vrf_sign<C: CSPRNG>(
        &self,
        rng: &mut C,
        data: sr25519::VRFData<'_>,
    ) -> Result<Vec<u8>, ()> {
        match self {
            Self::Ed25519(_) | Self::Ecdsa(_) => Err(()),
            Self::Sr25519(kp) => Ok(kp.vrf_sign(rng, data)),
        }
    }
}

impl std::cmp::PartialEq<CryptoAlgo> for Keypair {
    fn eq(&self, other: &CryptoAlgo) -> bool {
        match (self, other) {
            (Self::Sr25519(_), CryptoAlgo::Sr25519) => true,
            (Self::Ed25519(_), CryptoAlgo::Ed25519) => true,
            (Self::Ecdsa(_), CryptoAlgo::Ecdsa) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum PublicKey {
    Sr25519(sr25519::PublicKey),
    Ed25519(ed25519::PublicKey),
    Ecdsa(ecdsa::PublicKey),
}

impl PublicKey {
    pub fn from_bytes(algo: CryptoAlgo, key: &[u8]) -> Result<Self, ()> {
        match algo {
            CryptoAlgo::Sr25519 => sr25519::PublicKey::from_bytes(key)
                .map(Self::Sr25519)
                .map_err(|_| ()),
            CryptoAlgo::Ed25519 => ed25519::PublicKey::from_bytes(key)
                .map(Self::Ed25519)
                .map_err(|_| ()),
            CryptoAlgo::Ecdsa => ecdsa::PublicKey::from_bytes(key)
                .map(Self::Ecdsa)
                .map_err(|_| ()),
        }
    }

    pub fn verify<C: CSPRNG>(&self, rng: &mut C, msg: &[u8], sig: &[u8]) -> bool {
        match self {
            Self::Sr25519(pk) => {
                let mut array = [0; 64];
                array.copy_from_slice(&sig[..64]);
                pk.verify(rng, msg, &array)
            }
            Self::Ed25519(pk) => {
                let mut array = [0; 64];
                array.copy_from_slice(&sig[..64]);
                pk.verify(msg, &array)
            }
            Self::Ecdsa(pk) => {
                let mut array = [0; 65];
                array.copy_from_slice(&sig[..65]);
                pk.verify(msg, &array)
            }
        }
    }

    pub fn vrf_verify<C: CSPRNG>(
        &self,
        rng: &mut C,
        data: sr25519::VRFData<'_>,
        sig: &[u8],
    ) -> bool {
        match self {
            Self::Ed25519(_) | Self::Ecdsa(_) => false,
            Self::Sr25519(pk) => pk.verify_vrf(rng, data, sig),
        }
    }
}

impl From<Keypair> for PublicKey {
    fn from(kp: Keypair) -> Self {
        match kp {
            Keypair::Sr25519(kp) => Self::Sr25519(kp.into()),
            Keypair::Ed25519(kp) => Self::Ed25519(kp.into()),
            Keypair::Ecdsa(kp) => Self::Ecdsa(kp.into()),
        }
    }
}
