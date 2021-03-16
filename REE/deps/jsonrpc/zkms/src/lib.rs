//! This crate contains the JSON-RPC definition of the API

use jsonrpc_core::{BoxFuture, Error as RpcError, Result as RpcResult};
use jsonrpc_derive::rpc;

use zkms_common::{
    schnorrkel::{PublicKey, Signature},
    RequestError,
};

#[cfg_attr(feature = "client", rpc)]
#[cfg_attr(not(feature = "client"), rpc(server))]
pub trait ZKMS {
    #[rpc(name = "generateNew")]
    fn generate_new(&self, seed: Option<String>) -> BoxFuture<RpcResult<PublicKey>>;

    #[rpc(name = "getPublicKeys")]
    fn get_public_keys(&self) -> BoxFuture<RpcResult<Vec<PublicKey>>>;

    #[rpc(name = "signMessage")]
    fn sign_message(&self, public_key: PublicKey, msg: Vec<u8>) -> BoxFuture<RpcResult<Signature>>;
}

use derive_more::From;

#[derive(From, Debug)]
///This allows to properly go from a RequestError to an RpcError
/// without simplyfing the error exessively (ie making everything a string)
pub struct ErrorWrapper(RequestError);

impl Into<RpcError> for ErrorWrapper {
    fn into(self) -> RpcError {
        match self.0 {
            RequestError::InternalError(cause) => {
                let mut err = RpcError::internal_error();
                err.message = cause;
                err
            }
            RequestError::NoKeys(key) => {
                let hex = hex::encode(&key.to_bytes()[..]);
                let err = RpcError::invalid_params(format!("key `{}` was not recognized", hex));
                err
            }
        }
    }
}
