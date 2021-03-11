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
    fn recover_panic();
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

#[no_mangle]
pub extern "C" fn run() -> u32 {
    env_logger::init(); //can use any logger impl if you want

    /* ****************
        Place your logic from here....
        You could do it directly in this crate or separate it out to another crate so it's easier to test
        
        Below is some example code
    ****************** */
    
    let msg = b"been to trusted and back";
    
    let mut out = vec![0u8; msg.len()];

    let p0 = ParamTmpRef::new_input(&msg[..]);
    let p1 = ParamTmpRef::new_output(&mut out[..]);

    let mut op = Operation::new(p0, p1, ParamNone, ParamNone);
    if let Err(n) = invoke_command(0, &mut op) {
        return n;
    }
    
    assert_eq!(&msg[..], &out[..]);
    0
}
