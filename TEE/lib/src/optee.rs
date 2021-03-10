use ta_app::{close_session, open_session};
use zondee_utee::wrapper::{raw::TEE_Param, TEELogger, TEERng, TaErrorCode as Error};

//The signatures of the following functions are defined in the framework's rustee_ta.h file

#[no_mangle]
pub extern "C" fn RUSTEE_Create() -> u32 {
    TEELogger::install().expect("unable to set logger");

    trace!("Creating");

    // Only one instance is allowed to run by session
    if let Err(_) = open_session(TEERng::new_static()) {
        error!("[ERROR] can not create inner handler");
        Error::AccessDenied as _
    } else {
        info!("[INFO] *****Session created");
        0
    }
}

#[no_mangle]
pub extern "C" fn RUSTEE_Destroy() -> () {
    trace!("Destroying");
}

#[no_mangle]
pub extern "C" fn RUSTEE_OpenSession(
    _param_types: u32,
    _params: &mut [TEE_Param; 4],
    _session_context: *const u8,
) -> u32 {
    trace!("Opening session");

    0
}

#[no_mangle]
pub extern "C" fn RUSTEE_CloseSession(_session_context: *const u8) -> () {
    trace!("Closing session");
    close_session();
}

#[no_mangle]
pub extern "C" fn RUSTEE_InvokeCommand(
    _session_context: *const u8,
    cmd_id: u32,
    param_types: u32,
    params: &mut [TEE_Param; 4],
) -> u32 {
    trace!("Invoked command");

    super::invoke_command(cmd_id, param_types, params)
}
