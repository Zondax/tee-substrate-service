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
    // Calls he host client service passing the handler that should be used for requests
    host_app::start_service(optee_handler::Handler::default());
    0
}
