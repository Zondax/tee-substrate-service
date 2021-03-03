//! This crate contains the JSON-RPC definition of the API

use jsonrpc_core::{BoxFuture, Result as RpcResult};
use jsonrpc_derive::rpc;

use zkms_common::schnorrkel::{PublicKey, Signature};

#[rpc(server)]
pub trait ZKMS {
    #[rpc(name = "generateNew")]
    fn generate_new(&self, seed: Option<String>) -> BoxFuture<RpcResult<PublicKey>>;

    #[rpc(name = "getPublicKeys")]
    fn get_public_keys(&self) -> BoxFuture<RpcResult<Vec<PublicKey>>>;

    #[rpc(name = "signMessage")]
    fn sign_message(&self, public_key: PublicKey, msg: Vec<u8>) -> BoxFuture<RpcResult<Signature>>;
}
