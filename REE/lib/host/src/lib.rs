//! This crate glues a bunch of stuff together
//!
//! It starts the chosen service (dependency and code) and feeds it to the logic in `host_app`, aswell as the
//! default handler for the requests

#![no_builtins]

mod optee_handler;

use optee_common::{CommandId, TeeError};
use zondee_teec::wrapper::{raw, Operation, Param};

use host_app;

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
    //create tokio runtime for the application
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .expect("unable to initialize tokio runtime");

    rt.block_on(async move {
        //start the service
        let service = host_jsonrpc::start_service("127.0.0.1:39946").await;

        //call the host service that retrieves requests and handles them with the appropriate handler
        host_app::start_service(service, optee_handler::Handler::default()).await;
        0
    })
}
