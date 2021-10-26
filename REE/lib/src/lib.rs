//! This crate glues a bunch of stuff together
//!
//! It starts the chosen service (dependency and code) and feeds it to the logic in `host_app`, aswell as the
//! default handler for the requests

#![feature(extend_key_value_attributes)]
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

    let mut rt_builder = tokio::runtime::Builder::new();
    rt_builder.enable_all();

    cfg_if::cfg_if! {
        if #[cfg(feature = "ci")] {
            rt_builder.threaded_scheduler();
        } else {
            rt_builder.basic_scheduler();
        }
    }

    //create tokio runtime for the application
    let mut rt = rt_builder
        .build()
        .expect("unable to initialize tokio runtime");

    rt.block_on(async move {
        info!("starting ductile service...");
        //start the service
        let service = host_ductile::start_service(("0.0.0.0", PORT)).await;
        info!("ductile service started! forwarding to handler...");

        cfg_if::cfg_if! {
            if #[cfg(feature = "ci")] {
                let maybe_ci = ci::execute_tests(("localhost", PORT));
            } else {
                let maybe_ci = futures::future::pending::<()>();
            }
        }

        //spawn the host service that retrieves requests and handles them with the appropriate handler
        let service = tokio::spawn(host_app::start_service(
            service,
            optee_handler::Handler::default(),
        ));

        futures::pin_mut!(maybe_ci);
        futures::pin_mut!(service);

        futures::future::select(maybe_ci, service).await;

        0
    })
}

#[cfg(feature = "ci")]
mod ci;

mod utils;
