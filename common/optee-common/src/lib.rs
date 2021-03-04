//! Common types definitions to be used by host and ta
#![no_std]

extern crate no_std_compat as std;

use std::prelude::v1::*;

mod tee_error;
pub use tee_error::{TeeError, TeeErrorCode};

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum CommandId {
    GenerateNew,
    GetKeys,
    SignMessage,
}

impl std::convert::TryFrom<u32> for CommandId {
    type Error = ();

    fn try_from(cmd: u32) -> Result<Self, ()> {
        match cmd {
            0 => Ok(CommandId::GenerateNew),
            1 => Ok(CommandId::GetKeys),
            2 => Ok(CommandId::SignMessage),
            _ => Err(()),
        }
    }
}

impl Into<u32> for CommandId {
    fn into(self) -> u32 {
        self as _
    }
}

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
