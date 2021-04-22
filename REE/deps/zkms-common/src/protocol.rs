pub use sp_core::{ecdsa, ed25519, sr25519};
pub use sp_keystore::vrf::{VRFSignature, VRFTranscriptData};

/// Request handler interface
pub trait HandleRequest: Send + Sync {
    /// process a request
    fn process_request(&self, request: RequestMethod) -> Result<RequestResponse, RequestError>;
}

///Represents the type of algorithm to use for the key generation
#[cfg_attr(feature = "serde_", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub enum CryptoAlgo {
    Sr25519,
    Ed25519,
    Ecdsa,
}

///Represents the HasKey request arguments
#[cfg_attr(feature = "serde_", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub struct HasKeysPair {
    pub key_type: [u8; 4],
    pub public_key: Vec<u8>,
}

#[cfg_attr(feature = "serde_", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub enum RequestMethod {
    GenerateNew {
        algo: CryptoAlgo,
        key_type: [u8; 4],
    },
    GetPublicKeys {
        algo: CryptoAlgo,
        key_type: [u8; 4],
    },
    HasKeys {
        pairs: Vec<HasKeysPair>,
    },
    SignMessage {
        key_type: [u8; 4],
        public_key: Vec<u8>,
        msg: Vec<u8>,
    },
    VrfSign {
        key_type: [u8; 4],
        public_key: sr25519::Public,
        transcript_data: VRFTranscriptData,
    },
}

#[cfg_attr(feature = "serde_", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug)]
pub enum RequestResponse {
    GenerateNew {
        public_key: Vec<u8>,
    },
    GetPublicKeys {
        keys: Vec<Vec<u8>>,
    },
    HasKeys {
        all: bool,
    },
    SignMessage {
        signature: Vec<u8>,
    },
    VrfSign {
        signature: VRFSignature,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error("internal error caused by: {0}")]
    InternalError(String),
    #[error("no keys match the given key `{0:x?}`")]
    NoKeys(Vec<u8>),
}

impl From<String> for RequestError {
    fn from(from: String) -> Self {
        Self::InternalError(from)
    }
}
