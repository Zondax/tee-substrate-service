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
    fn recover_panic();
}

pub(crate) fn invoke_command<A: Param, B: Param, C: Param, D: Param>(
    id: CommandId,
    op: &mut Operation<A, B, C, D>,
) -> Result<(), TeeError> {
    let res = unsafe { invoke_optee_command(id as u32, op.as_mut_ptr()) };
    if res == 0 {
        Ok(())
    } else {
        let err = TeeError::from_raw_error(res);
        if let optee_common::TeeErrorCode::TargetDead = err.kind() {
            unsafe {
                recover_panic();
            }
        }
        error!("An error occured when invoking command: {}", err.message());
        Err(err)
    }
}

#[no_mangle]
pub extern "C" fn run() -> u32 {
    const PORT: u16 = 39946;

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
        let service = host_jsonrpc::start_service(("0.0.0.0", PORT)).await;
        info!("jsonrpc service started! forwarding to handler...");

        cfg_if::cfg_if! {
            if #[cfg(feature = "ci")] {
                let maybe_ci = ci::execute_tests(("localhost", PORT));
            } else {
                let maybe_ci = futures::future::pending::<()>();
            }
        }

        //call the host service that retrieves requests and handles them with the appropriate handler
        let service = host_app::start_service(service, optee_handler::Handler::default());

        futures::pin_mut!(maybe_ci);
        futures::pin_mut!(service);

        futures::future::select(maybe_ci, service).await;

        0
    })
}

#[cfg(feature = "ci")]
mod ci;
