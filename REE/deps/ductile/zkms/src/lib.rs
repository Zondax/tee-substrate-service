pub use sp_core::{crypto, ecdsa, ed25519, sr25519};
pub use sp_keystore::{
    vrf::{VRFSignature, VRFTranscriptData, VRFTranscriptValue},
    Error as KeystoreError,
};

use sp_core::crypto::{CryptoTypePublicPair, KeyTypeId};

use serde::{Deserialize, Serialize};

#[macro_use]
extern crate tracing;

#[derive(Clone, Debug, Serialize, Deserialize)]
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
        msg: Vec<u8>,
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
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RemoteKeystoreResponse {
    Sr25519PublicKeys(Vec<sr25519::Public>),
    Sr25519GenerateNew(Result<sr25519::Public, KeystoreError>),
    Ed25519PublicKeys(Vec<ed25519::Public>),
    Ed25519GenerateNew(Result<ed25519::Public, KeystoreError>),
    EcdsaPublicKeys(Vec<ecdsa::Public>),
    EcdsaGenerateNew(Result<ecdsa::Public, KeystoreError>),
    InsertUnknown(Result<(), ()>),
    SupportedKeys(Result<Vec<CryptoTypePublicPair>, KeystoreError>),
    Keys(Result<Vec<CryptoTypePublicPair>, KeystoreError>),
    HasKeys(bool),
    SignWith(Result<Vec<u8>, KeystoreError>),
    SignWithAny(Result<(CryptoTypePublicPair, Vec<u8>), KeystoreError>),
    SignWithAll(Result<Vec<Result<Vec<u8>, KeystoreError>>, ()>),
    Sr25519VrfSign(Result<VRFSignature, KeystoreError>),
}
