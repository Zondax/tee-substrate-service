//! This crate glues a bunch of stuff together
//!
//! It starts the chosen service (dependency and code) and feeds it to the logic in `host_app`, aswell as the
//! default handler for the requests

#![no_builtins]

#[macro_use]
extern crate log;

mod optee_handler;

use optee_common::{CommandId, TeeError};
use zondee_teec::wrapper::{raw, Operation, Param};

extern "C" {
    fn invoke_optee_command(command_id: u32, op: *mut raw::TEEC_Operation) -> u32;
}

pub(crate) fn invoke_command<A: Param, B: Param, C: Param, D: Param>(
    id: CommandId,
    op: &mut Operation<A, B, C, D>,
) -> Result<(), TeeError> {
    let res = unsafe { invoke_optee_command(id as u32, op.as_mut_ptr()) };
    if res == 0 {
        Ok(())
    } else {
        Err(TeeError::from_raw_error(res))
    }
}

#[no_mangle]
pub extern "C" fn run() -> u32 {
    env_logger::init();

    //create tokio runtime for the application
    let mut rt = tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_io()
        .build()
        .expect("unable to initialize tokio runtime");

    rt.block_on(async move {
        info!("starting jsonrpc service...");
        //start the service
        let service = host_jsonrpc::start_service("0.0.0.0:39946").await;
        info!("jsonrpc service started! forwarding to handler...");
        //call the host service that retrieves requests and handles them with the appropriate handler
        host_app::start_service(service, optee_handler::Handler::default()).await;
        0
    })
}
