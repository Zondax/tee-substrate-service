#![no_main]
use libfuzzer_sys::fuzz_target;

use std::convert::TryFrom;

extern crate ta_app;
use ta_app::TaApp;

extern crate optee_common;
use optee_common::{CommandId, HandleTaCommand};

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let mut app = TaApp::default();

    //4 bytes cmd id minimum
    if data.len() < 4 {
        return;
    }

    let cmd_id = {
        let mut array = [0; 4];
        array.copy_from_slice(&data[..4]);
        u32::from_ne_bytes(array)
    };

    if let Ok(cmd) = CommandId::try_from(cmd_id) {
        let mut out = vec![0; 100 * 1024];
        let _ = app.process_command(cmd, &data[4..], &mut out);
    }
});
