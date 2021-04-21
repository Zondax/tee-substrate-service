use sp_core::{
    crypto::{CryptoTypePublicPair, KeyTypeId},
    ecdsa, ed25519, sr25519,
};
use sp_keystore::{Error, vrf::{VRFSignature, VRFTranscriptData}};

use serde::{Serialize, Deserialize};

#[macro_use]
extern crate tracing;

#[derive(Debug, Serialize, Deserialize)]
pub enum RemoteKeystore {
    Sr25519PublicKeys(KeyTypeId),
    Sr25519GenerateNew {
        id: KeyTypeId,
        seed: Option<String>,
    },
    Ed25519PublicKeys(KeyTypeId),
    Ed25519GenerateNew {
        id: KeyTypeId,
        seed: Option<String>,
    },
    EcdsaPublicKeys(KeyTypeId),
    EcdsaGenerateNew {
        id: KeyTypeId,
        seed: Option<String>,
    },
    InsertUnknown {
        id: KeyTypeId,
        suri: String,
        public: Vec<u8>,
    },
    SupportedKeys {
        id: KeyTypeId,
        keys: Vec<CryptoTypePublicPair>,
    },
    Keys(KeyTypeId),
    HasKeys(Vec<(Vec<u8>, KeyTypeId)>),
    SignWith {
        id: KeyTypeId,
        key: CryptoTypePublicPair,
        msg: Vec<u8>
    },
    SignWithAny {
        id: KeyTypeId,
        keys: Vec<CryptoTypePublicPair>,
        msg: Vec<u8>,
    },
    SignWithAll {
        id: KeyTypeId,
        keys: Vec<CryptoTypePublicPair>,
        msg: Vec<u8>,
    },
    Sr25519VrfSign {
        key_type: KeyTypeId,
        public: sr25519::Public,
        transcript_data: VRFTranscriptData,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RemoteKeystoreResponse {
    Sr25519PublicKeys(Vec<sr25519::Public>),
    Sr25519GenerateNew(Result<sr25519::Public, Error>),
    Ed25519PublicKeys(Vec<ed25519::Public>),
    Ed25519GenerateNew(Result<ed25519::Public, Error>),
    EcdsaPublicKeys(Vec<ecdsa::Public>),
    EcdsaGenerateNew(Result<ecdsa::Public, Error>),
    InsertUnknown(Result<(), ()>),
    SupportedKeys(Result<Vec<CryptoTypePublicPair>, Error>),
    Keys(Result<Vec<CryptoTypePublicPair>, Error>),
    HasKeys(bool),
    SignWith(Result<Vec<u8>, Error>),
    SignWithAny(Result<(CryptoTypePublicPair, Vec<u8>), Error>),
    SignWithAll(Result<Vec<Result<Vec<u8>, Error>>, ()>),
    Sr25519VrfSign(Result<VRFSignature, Error>)
}
