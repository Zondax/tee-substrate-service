//! Common types definitions to be used by host and ta
#![no_std]

mod tee_error;
pub use tee_error::{TeeError, TeeErrorCode};

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum CommandId {
    Unknown,
}

impl From<u32> for CommandId {
    fn from(cmd: u32) -> Self {
        match cmd {
            _ => CommandId::Unknown,
        }
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
