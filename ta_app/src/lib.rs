#![no_std]

use core::cell::{Ref, RefCell, RefMut};

use optee_common::{CommandId, HandleTaCommand, TeeErrorCode as Error};

#[macro_use]
extern crate log;

#[derive(Default)]
pub struct TaApp {}

// This is safe because all request are serialized by the TA framework
unsafe impl Sync for TaApp {}

type InnerHandler<T> = RefCell<Option<T>>;

/// Main TA request handler which wrapps any type that implements the HandleTaCommand Trait
struct TaHandler<T>(InnerHandler<T>);

// This is safe because the ta framework serializes all of the incoming requests so that only one is
// processed at time
unsafe impl<T: HandleTaCommand + Sync + Default> Sync for TaHandler<T> {}

// The privite handler for processing client commands
static TA_HANDLER: TaHandler<TaApp> = TaHandler(RefCell::new(None));

impl HandleTaCommand for TaApp {
    fn process_command(
        &mut self,
        cmd_id: CommandId,
        mut input: &[u8],
        output: &mut [u8],
    ) -> Result<(), Error> {
        todo!()
    }
}

const U64_SIZE: usize = core::mem::size_of::<u64>();
impl TaApp {
    ///Reads an u64 from the slice, advancing it
    fn read_and_advance_u64(&self, slice: &mut &[u8]) -> Result<u64, Error> {
        if slice.len() < U64_SIZE {
            return Err(Error::OutOfMemory);
        }

        //read and advance slice
        let mut tmp = [0; U64_SIZE];
        tmp.copy_from_slice(&slice[..U64_SIZE]);
        *slice = &slice[U64_SIZE..];

        Ok(u64::from_le_bytes(tmp))
    }

    ///Makes sure the input and output slice have enough length
    const fn check_mem(cmd: CommandId, in_len: usize, out_len: usize) -> Result<(), Error> {
        match cmd {
            CommandId::Unknown => Err(Error::NotSupported),
        }
    }
}

pub fn open_session() -> Result<(), ()> {
    // At this point no handler should have been created
    // Only one instance is allowed, so we create our command handler on each new session.
    TA_HANDLER.0.borrow_mut().replace(TaApp::default());
    Ok(())
}

pub fn close_session() {
    // Once the client is done, the TA session is closed, dropping our handler
    let _ = TA_HANDLER.0.borrow_mut().take();
}

pub fn borrow_mut_app<'a>() -> RefMut<'a, Option<impl HandleTaCommand + 'static>> {
    trace!("Getting TA_app mut handler");
    TA_HANDLER.0.borrow_mut()
}

pub fn borrow_app<'a>() -> Ref<'a, Option<impl HandleTaCommand + 'static>> {
    trace!("Getting TA_app handler");
    TA_HANDLER.0.borrow()
}

#[cfg(test)]
mod tests {
    use super::*;
}
