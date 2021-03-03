use ta_app::{close_session, open_session};
use zondee_utee::wrapper::{TEERng, TEELogger, TaErrorCode as Error};

#[no_mangle]
pub extern "C" fn TA_CreateEntryPoint() -> u32 {
    TEELogger::install().expect("unable to set logger");

    trace!("CreateEntryPoint has been called");
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
pub extern "C" fn TA_DestroyEntryPoint() -> () {
    trace!("Destroying entry point");
}

#[no_mangle]
pub extern "C" fn TA_CloseSessionEntryPoint(_session_context: *const u8) -> () {
    trace!("Clossing session");
    close_session();
}