//! Common types definitions to be used by host and ta
#![no_std]

extern crate no_std_compat as std;

use std::prelude::v1::*;

mod tee_error;
pub use tee_error::{TeeError, TeeErrorCode};

mod types;
pub use types::*;

// TODO trait should be more generic. We might have different type of parameters or None at all.
pub trait HandleTaCommand {
    fn process_command(
        &mut self,
        cmd_id: CommandId,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<(), TeeErrorCode>;
}

mod serde;
pub use serde::*;
