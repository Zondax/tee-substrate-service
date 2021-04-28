use super::*;

use crate::util::CSPRNG;
use optee_common::CryptoAlgo;

mod ecdsa;
mod ed25519;
mod sr25519;
pub use sr25519::VRFData;

/// Will contain secrets set during compilation (for initial provisioning)
///
/// Temporary
mod secrets;

fn str_to_key_type(s: &str) -> [u8; 4] {
    let mut array = [0; 4];
    for (i, b) in s
        .as_bytes()
        .iter()
        .chain(core::iter::once(&0u8).cycle())
        .enumerate()
        .take(4)
    {
        array[i] = *b;
    }

    array
}

fn populate_set(
    seed: &str,
    key_types: &str,
    algo: CryptoAlgo,
    map: &mut HashMap<[u8; 4], Vec<Keypair>, crate::util::hasher::Builder>,
) {
    if let Ok(seed) = hex::decode(&seed[2..]) {
        for kt in key_types.split_whitespace() {
            let kt = str_to_key_type(kt);

            if let Some(kp) = Keypair::from_bytes(&seed, algo) {
                map.entry(kt).or_default().push(kp)
            }
        }
    }
}

pub fn default_set() -> HashMap<[u8; 4], Vec<Keypair>, crate::util::hasher::Builder> {
    let mut map: HashMap<_, Vec<Keypair>, _> = HashMap::with_hasher(Default::default());

    populate_set(
        secrets::SR_SECRET
            .unwrap_or("0x4ed8d4b17698ddeaa1f1559f152f87b5d472f725ca86d341bd0276f1b61197e2"),
        secrets::SR_KEY_TYPES.unwrap_or("babe imon audi"),
        CryptoAlgo::Sr25519,
        &mut map,
    );

    populate_set(
        secrets::ED_SECRET
            .unwrap_or("0x4ed8d4b17698ddeaa1f1559f152f87b5d472f725ca86d341bd0276f1b61197e2"),
        secrets::ED_KEY_TYPES.unwrap_or("gran"),
        CryptoAlgo::Ed25519,
        &mut map,
    );

    populate_set(
        secrets::EC_SECRET
            .unwrap_or("0x4ed8d4b17698ddeaa1f1559f152f87b5d472f725ca86d341bd0276f1b61197e2"),
        secrets::EC_KEY_TYPES.unwrap_or(""),
        CryptoAlgo::Ecdsa,
        &mut map,
    );

    map
}

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

    pub fn from_bytes(secret: &[u8], algo: CryptoAlgo) -> Option<Self> {
        match algo {
            CryptoAlgo::Sr25519 => sr25519::Keypair::from_bytes(secret).map(Self::Sr25519),
            CryptoAlgo::Ed25519 => ed25519::Keypair::from_bytes(secret).map(Self::Ed25519),
            CryptoAlgo::Ecdsa => ecdsa::Keypair::from_bytes(secret).map(Self::Ecdsa),
        }
    }

    pub fn public_bytes(&self) -> &[u8] {
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

    pub fn to_public_key(&self) -> PublicKey {
        self.into()
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

    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> bool {
        match self {
            Self::Sr25519(pk) => {
                let mut array = [0; 64];
                array.copy_from_slice(&sig[..64]);
                pk.verify(msg, &array)
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

    pub fn vrf_verify(&self, data: sr25519::VRFData<'_>, sig: &[u8]) -> bool {
        match self {
            Self::Ed25519(_) | Self::Ecdsa(_) => false,
            Self::Sr25519(pk) => pk.verify_vrf(data, sig),
        }
    }
}

impl From<&Keypair> for PublicKey {
    fn from(kp: &Keypair) -> Self {
        match kp {
            Keypair::Sr25519(kp) => Self::Sr25519(kp.into()),
            Keypair::Ed25519(kp) => Self::Ed25519(kp.into()),
            Keypair::Ecdsa(kp) => Self::Ecdsa(kp.into()),
        }
    }
}
