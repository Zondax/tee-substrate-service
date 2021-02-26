#![deny(
    rust_2018_idioms,
    trivial_casts,
    unused_lifetimes,
    unused_qualifications
)]
use rand::Rng;
use zkms_common::{HandleRequest, RequestMethod, RequestResponse};

pub fn start_service(_handler: impl HandleRequest + 'static) {
    //nothing yet

    //ref to https://github.com/gnunicorn/substrate-remote-signer-example
    // this app should talk to the substrate node, aswell as handle the requests sent to it

    //initialize listener to retrieve requests from
    // for each request transaform and pass to `handler`
    // process response and reply
    todo!()
}
