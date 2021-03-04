//! Common definitions between host and outside world
//!
//! This crate is most likely gonna be used only by implementations of the protocol and by `host`

#![deny(
    clippy::expect_used,
    //missing_debug_implementations,
    rust_2018_idioms,
    trivial_casts,
    //unsafe_code,
    unused_lifetimes
)]

pub mod protocol;
pub use protocol::{HandleRequest, RequestError, RequestMethod, RequestResponse};

pub use schnorrkel;
