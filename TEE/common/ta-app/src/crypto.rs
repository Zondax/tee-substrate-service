use crate::util::CSPRNG;

use super::*;

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
            Keypair::Sr25519(kp) => kp.public(),
            Keypair::Ed25519(kp) => kp.public(),
            Keypair::Ecdsa(kp) => kp.public(),
        }
    }

    pub fn sign<C: CSPRNG>(&self, rng: &mut C, msg: &[u8]) -> Vec<u8> {
        match self {
            Keypair::Sr25519(kp) => kp.sign(rng, msg).to_vec(),
            Keypair::Ed25519(kp) => kp.sign(msg).to_vec(),
            Keypair::Ecdsa(kp) => kp.sign(msg).to_vec(),
        }
    }

    pub fn vrf_sign<C: CSPRNG>(
        &self,
        rng: &mut C,
        data: sr25519::VRFData<'_>,
    ) -> Result<Vec<u8>, ()> {
        match self {
            Keypair::Ed25519(_) | Keypair::Ecdsa(_) => Err(()),
            Keypair::Sr25519(kp) => Ok(kp.vrf_sign(rng, data)),
        }
    }
}
