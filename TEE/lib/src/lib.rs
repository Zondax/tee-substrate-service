#![no_std]
#![no_builtins]
#![cfg_attr(not(test), feature(alloc_error_handler))]

use optee_common::{CommandId, HandleTaCommand};
use ta_app::borrow_mut_app;
use zondee_utee::wrapper::{
    raw::{TEE_Param, TEE_PARAM_TYPES},
    ParamType, Parameters, TaErrorCode as Error,
};

/// This module contains the functions that will be called from the C library
///
/// The signature of the functions are found in the librustee_ta.h file
mod optee;

#[macro_use]
extern crate log;

use core::convert::TryFrom;

#[cfg(not(test))]
/// This module is used to provide lang items needed but not normally available in the TEE
///
/// This is, for example, the `panic_handler`, the `global_allocator` and the `alloc_error_handler`
mod lang_items {
    use core::panic::PanicInfo;
    use zondee_utee::wrapper::{utee_panic, TEEAllocator};

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        error!("[ERROR] TA Panic: {}", info);

        utee_panic(0)
    }

    #[global_allocator]
    static GLOBAL: TEEAllocator = TEEAllocator;

    #[alloc_error_handler]
    fn oom(_: core::alloc::Layout) -> ! {
        utee_panic(1)
    }
}

pub fn invoke_command(cmd_id: u32, param_types: u32, parameters: &mut [TEE_Param; 4]) -> u32 {
    let mut params = Parameters::from_raw(parameters, param_types);

    // This check would depend on the opretion defined by cmd_id
    // We might decide to limit the params to be only two, an input and output
    // slice.
    let expected_param_types = TEE_PARAM_TYPES(
        ParamType::MemRefInput as u32,
        ParamType::MemRefOutput as u32,
        ParamType::None as u32,
        ParamType::None as u32,
    );
    if param_types != expected_param_types {
        error!("[ERROR] Bad parameters");
        return Error::BadParameters as _;
    }

    let mut imemref = unsafe {
        params
            .0
            .as_memref()
            .expect("this is safe, the type was previously check")
    };
    let mut omemref = unsafe {
        params
            .1
            .as_memref()
            .expect("this is safe, the type was previously check")
    };

    let cmd = match CommandId::try_from(cmd_id) {
        Ok(cmd) => cmd,
        Err(_) => return Error::NotSupported as _,
    };

    // The inner handler could have persistance data or state that is required along the execution of the program
    // so instead of creating a handler on every command_invocation, we create the handler when the session is opened.
    // Such session remains open until the TEEC closes it, at this point the handler must be already created.
    borrow_mut_app()
        .as_mut()
        .map_or(Error::ItemNotFound as u32, |ta_handler| {
            if let Err(e) = ta_handler.process_command(cmd, imemref.buffer(), omemref.buffer()) {
                error!("[ERROR] processing command failure: {:?}", e);
                e as _
            } else {
                0
            }
        })
}
