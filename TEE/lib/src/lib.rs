#![no_std]
#![no_builtins]
#![cfg_attr(not(test), feature(alloc_error_handler))]

use zondee_utee::wrapper::{
    raw::{TEE_Param, TEE_PARAM_TYPES},
    ParamType, Parameters, TaErrorCode as Error,
};

#[macro_use]
extern crate log;

mod optee;

#[cfg(not(test))]
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

fn invoke_command(_cmd_id: u32, param_types: u32, parameters: &mut [TEE_Param; 4]) -> u32 {
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
            .expect("this is safe, the type was previously checked")
    };
    let mut omemref = unsafe {
        params
            .1
            .as_memref()
            .expect("this is safe, the type was previously checked")
    };

    /* **********
        Place your logic from here...
        You could do it directly in this crate or separate it out to another crate so it's easier to test

        Below is some example code
    ************ */

    //let's ignore the cmd_id since we only accept 1 command
    //which is the echo command

    let input = imemref.buffer();
    let out = omemref.buffer();

    if input.len() > out.len() {
        return Error::BadFormat as _;
    }

    out.copy_from_slice(&input[..input.len()]);

    0
}
