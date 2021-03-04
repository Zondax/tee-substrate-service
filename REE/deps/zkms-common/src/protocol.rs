use schnorrkel::{keys::PublicKey, sign::Signature};

/// Request handler interface
pub trait HandleRequest: Send + Sync {
    /// process a request
    fn process_request(&self, request: RequestMethod) -> Result<RequestResponse, RequestError>;
}

#[cfg_attr(feature = "serde_", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub enum RequestMethod {
    GenerateNew { seed: Option<String> },
    GetPublicKeys,
    SignMessage { public_key: PublicKey, msg: Vec<u8> },
}

#[cfg_attr(feature = "serde_", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug)]
pub enum RequestResponse {
    GenerateNew { public_key: PublicKey },
    GetPublicKeys { keys: Vec<PublicKey> },
    SignMessage { signature: Signature },
}

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error("internal error caused by: {0}")]
    InternalError(String),
    #[error("no keys match the given key `{0:x?}`")]
    NoKeys(PublicKey),
}

impl From<String> for RequestError {
    fn from(from: String) -> Self {
        Self::InternalError(from)
    }
}
