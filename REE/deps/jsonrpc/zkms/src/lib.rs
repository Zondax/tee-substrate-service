//! This crate contains the JSON-RPC definition of the API

use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_derive::rpc;

use zkms_common::schnorrkel::{PublicKey, Signature};

#[rpc(server)]
pub trait ZKMS {
    #[rpc(name = "generateNew", returns = "PublicKey")]
    fn generate_new(&self, seed: Option<String>) -> BoxFuture<Result<PublicKey>>;

    #[rpc(name = "getPublicKeys", returns = "Vec<PublicKey>")]
    fn get_public_keys(&self) -> BoxFuture<Result<Vec<PublicKey>>>;

    #[rpc(name = "signMessage", returns = "Result<Signature>")]
    fn sign_message(&self, public_key: PublicKey, msg: Vec<u8>) -> BoxFuture<Result<Signature>>;
}
