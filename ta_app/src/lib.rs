#![no_std]

use core::cell::{Ref, RefCell, RefMut};

use optee_common::{CommandId, HandleTaCommand, TeeErrorCode as Error};
use rand_core::{CryptoRng, RngCore};
use schnorrkel::keys::{Keypair, PUBLIC_KEY_LENGTH};

#[macro_use]
extern crate log;

pub struct TaApp<'r> {
    keys: [Option<Keypair>; 1], //we'll have to figure later how to expand this
    rng: &'r mut dyn CSPRNG,    //the rng provider
}

// This is safe because all request are serialized by the TA framework
unsafe impl<'r> Sync for TaApp<'r> {}

type InnerHandler<T> = RefCell<Option<T>>;

/// Main TA request handler which wrapps any type that implements the HandleTaCommand Trait
struct TaHandler<T>(InnerHandler<T>);

// This is safe because the ta framework serializes all of the incoming requests so that only one is
// processed at time
unsafe impl<T: HandleTaCommand + Sync> Sync for TaHandler<T> {}

trait CSPRNG: CryptoRng + RngCore {}
impl<R: CryptoRng + RngCore> CSPRNG for R {}

// The privite handler for processing client commands
static TA_HANDLER: TaHandler<TaApp<'static>> = TaHandler(RefCell::new(None));

impl<'r> HandleTaCommand for TaApp<'r> {
    fn process_command(
        &mut self,
        cmd_id: CommandId,
        mut input: &[u8],
        output: &mut [u8],
    ) -> Result<(), Error> {
        trace!("Processing CMD {:?}", cmd_id);

        Self::check_mem(cmd_id, input, &output)?;

        match cmd_id {
            CommandId::GenerateNew => {
                let seed_len = Self::read_and_advance_u64(&mut input)? as _;
                match seed_len {
                    0 => {
                        let keypair = Keypair::generate_with(&mut self.rng);
                        let pk_bytes = keypair.public.to_bytes();

                        //store keypair
                        self.keys[0].replace(keypair);

                        //write to output
                        output[..PUBLIC_KEY_LENGTH].copy_from_slice(&pk_bytes[..]);
                    }
                    len => {
                        let seed =
                            core::str::from_utf8(&input[..len]).map_err(|_| Error::BadFormat)?;

                        todo!("private key with seed")
                    }
                }

                Ok(())
            }
            CommandId::GetKeys => {
                todo!()
            }
            CommandId::SignMessage => {
                todo!()
            }
        }
    }
}

const U64_SIZE: usize = core::mem::size_of::<u64>();
impl<'r> TaApp<'r> {
    ///Reads an u64 from the slice, advancing it
    fn read_and_advance_u64(slice: &mut &[u8]) -> Result<u64, Error> {
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
    fn check_mem(cmd: CommandId, mut input: &[u8], mut out: &[u8]) -> Result<(), Error> {
        match cmd {
            CommandId::GenerateNew => {
                let len = Self::read_and_advance_u64(&mut input)?;

                let input = input.len() >= len as _; //this already takes into account the initial len
                let out = out.len() >= PUBLIC_KEY_LENGTH;

                if input && out {
                    Ok(())
                } else {
                    Err(Error::OutOfMemory)
                }
            }
            CommandId::GetKeys => {
                todo!()
            }
            CommandId::SignMessage => {
                todo!()
            }
        }
    }
}

impl<'r> TaApp<'r> {
    pub fn with_rng<R: CryptoRng + RngCore + 'r>(rng: &'r mut R) -> Self {
        Self {
            rng: rng as _,
            keys: [None],
        }
    }
}

pub fn open_session<R: CryptoRng + RngCore + 'static>(rng: &'static mut R) -> Result<(), ()> {
    // At this point no handler should have been created
    // Only one instance is allowed, so we create our command handler on each new session.
    TA_HANDLER.0.borrow_mut().replace(TaApp::with_rng(rng));
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
    extern crate std;
    use super::*;

    #[test]
    fn get_random_key() {
        let mut rng = rand_core::OsRng;
        let mut app = TaApp::with_rng(&mut rng);

        let input = 0u64.to_le_bytes();
        let mut output = [0; PUBLIC_KEY_LENGTH];

        app.process_command(CommandId::GenerateNew, &input[..], &mut output)
            .expect("shouldn't fail");

        let valid_pk = schnorrkel::PublicKey::from_bytes(&output).expect("not a valid public key");
        std::dbg!(valid_pk);
    }
}
