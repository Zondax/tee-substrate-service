#![no_main]
use libfuzzer_sys::fuzz_target;

use std::convert::TryFrom;

extern crate ta_app;
use ta_app::TaApp;

extern crate optee_common;
use optee_common::{CommandId, HandleTaCommand};

#[derive(Debug, arbitrary::Arbitrary)]
struct Command {
    cmd: CommandId,
    input: Vec<u8>,
}

fuzz_target!(|data: Command| {
    // fuzzed code goes here
    let mut app = TaApp::default();

    let mut out = vec![0; 100 * 1024];
    let _ = app.process_command(data.cmd, data.input.as_slice(), &mut out);
});
