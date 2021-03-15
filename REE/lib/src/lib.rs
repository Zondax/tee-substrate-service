//! This crate glues a bunch of stuff together
//!
//! It starts the chosen service (dependency and code) and feeds it to the logic in `host_app`, aswell as the
//! default handler for the requests

#![no_builtins]

#[macro_use]
extern crate log;

use zondee_teec::wrapper::{raw, Operation, Param, ParamNone, ParamTmpRef};

extern "C" {
    fn invoke_optee_command(command_id: u32, op: *mut raw::TEEC_Operation) -> u32;
}

pub(crate) fn invoke_command<A: Param, B: Param, C: Param, D: Param>(
    id: u32,
    op: &mut Operation<A, B, C, D>,
) -> Result<(), u32> {
    let res = unsafe { invoke_optee_command(id, op.as_mut_ptr()) };
    if res == 0 {
        Ok(())
    } else {
        error!("An error occured when invoking command: {}", res);
        Err(res)
    }
}

mod logic;

#[no_mangle]
#[cfg(not(feature = "ci"))]
pub extern "C" fn run() -> u32 {
    env_logger::init(); //can use any logger impl if you want

    /* ****************
        Place your logic from here....
        You could do it directly in this crate or separate it out to another crate so it's easier to test

        Below is some example code
    ****************** */

    let msg = b"been to trusted and back";

    assert_eq!(logic::echo(msg), Ok(true));
    0
}

#[no_mangle]
#[cfg(feature = "ci")]
pub extern "C" fn run() -> u32 {
    env_logger::init();
    info!("TESTS STARTING");

    /*
     * Here lies the code for your automated tests!
     * */

    info!("[RUSTEE TEST #0]: START");
    let msg = b"hello ci";
    let result = logic::echo(msg).expect("[RUSTEE TEST #0]: ERROR");
    info!(
        "[RUSTEE TEST #0]: {}",
        if result { "SUCCESS" } else { "FAILURE" }
    );

    info!("TESTS FINISHED");
    0
}
