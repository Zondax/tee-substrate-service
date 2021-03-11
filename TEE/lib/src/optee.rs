use zondee_utee::wrapper::{TEELogger, TaErrorCode as Error};

//The signatures of the following functions are defined in the framework's rustee_ta.h file

#[no_mangle]
pub extern "C" fn RUSTEE_Create() -> u32 {
    TEELogger::install().expect("unable to set logger");
    trace!("Creating");
    
    /*
     *  Place your global initialization logic here
     * */

    0
}

#[no_mangle]
pub extern "C" fn RUSTEE_Destroy() -> () {
    trace!("Destroying");
    
    /*
     * Place your global cleanup logic here
     * */
}

#[no_mangle]
pub extern "C" fn RUSTEE_OpenSession(
    _param_types: u32,
    _params: &mut [TEE_Param; 4],
    _session_context: *const u8,
) -> u32 {
    trace!("Opening session");
    
    /*
     * Place session initialization logic here
     */

    0
}

#[no_mangle]
pub extern "C" fn RUSTEE_CloseSession(_session_context: *const u8) -> () {
    trace!("Closing session");
    
    /*
     * Place session cleanup logic here
     */
}

#[no_mangle]
pub extern "C" fn RUSTEE_InvokeCommand(
    _session_context: *const u8,
    cmd_id: u32,
    param_types: u32,
    params: &mut [TEE_Param; 4],
) -> u32 {
    trace!("Invoked command");

    /*
     * Place the application logic here
     */
    super::invoke_command(cmd_id, param_types, params)
}
