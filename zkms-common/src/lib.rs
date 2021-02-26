//! Common definitions
#![deny(
    clippy::expect_used,
    //missing_debug_implementations,
    rust_2018_idioms,
    trivial_casts,
    //unsafe_code,
    unused_lifetimes
)]

pub mod jsonrpc_request;
pub use jsonrpc_request::{HandleRequest, RequestMethod, RequestResponse};
