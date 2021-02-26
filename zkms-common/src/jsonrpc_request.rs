use schnorrkel::{keys::PublicKey, sign::Signature};

/// Request handler interface
pub trait HandleRequest: Send + Sync {
    /// process a request
    //fn process_request(&self, request: KeystoreRequest) -> Result<(), String>;
    fn process_request(&self, request: RequestMethod) -> Result<RequestResponse, String>;
}

#[cfg_attr(feature = "serde", derive(serde_::Deserialize, serde_::Serialize))]
#[derive(Debug, Clone)]
pub enum RequestMethod {
    GenerateNew { seed: Option<String> },
    GetPublicKeys,
    SignMessage { public_key: PublicKey, msg: Vec<u8> },
}

#[cfg_attr(feature = "serde", derive(serde_::Deserialize, serde_::Serialize))]
#[derive(Debug)]
pub enum RequestResponse {
    GenerateNew { public_key: PublicKey },
    GetPublicKeys { keys: Vec<PublicKey> },
    SignMessage { signature: Signature },
}
