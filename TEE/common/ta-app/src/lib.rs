#![no_std]

use core::cell::{Ref, RefCell, RefMut};

use optee_common::{
    CommandId, Deserialize, DeserializeOwned, HandleTaCommand, SerializeFixed,
    TeeErrorCode as Error,
};
use rand_core::{CryptoRng, RngCore};
use schnorrkel::{keys::Keypair, PublicKey, SecretKey, Signature};

mod util;
use util::CSPRNG;

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
        trace!("checked mem succesfully...");

        match cmd_id {
            CommandId::GenerateNew => {
                let seed: &str = Deserialize::deserialize(input).map_err(|_| Error::BadFormat)?;
                trace!("read seed_len={:?}", seed.len());

                match seed.len() {
                    0 => {
                        let keypair = Keypair::generate_with(&mut self.rng);
                        trace!("generated keypair; public={:x?}", keypair.public.to_bytes());

                        //write to output
                        keypair.public.serialize_fixed(output).unwrap();

                        //store keypair
                        self.keys[0].replace(keypair);
                    }
                    len => {
                        todo!("private key with seed")
                    }
                }

                Ok(())
            }
            CommandId::GetKeys => {
                todo!()
            }
            CommandId::SignMessage => {
                let public: PublicKey =
                    PublicKey::deserialize_owned(&input).map_err(|_| Error::BadFormat)?;
                trace!("read public key: {:x?}", public.to_bytes());

                util::advance_slice(&mut input, PublicKey::len()).unwrap();

                let msg: &[u8] = Deserialize::deserialize(&input).unwrap();
                trace!("read msg: {:x?}", msg);

                let secret = self
                    .find_associated_key(public)
                    .ok_or(Error::BadParameters)?;
                trace!("got key");

                let sig = self.sign(&secret, b"zondax", &msg, &public);
                trace!("signed!");

                sig.serialize_fixed(output).unwrap();
                trace!("signature copied to out");

                Ok(())
            }
        }
    }
}

impl<'r> TaApp<'r> {
    ///Makes sure the input and output slice have enough length
    fn check_mem(cmd: CommandId, mut input: &[u8], mut out: &[u8]) -> Result<(), Error> {
        match cmd {
            CommandId::GenerateNew => {
                let len = util::read_and_advance_u64(&mut input)?;

                let input = input.len() >= len as _;
                let out = out.len() >= PublicKey::len();

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
                //we can skip the public key here

                //attempt to read public_key, error if failed
                let _ = util::read_and_advance(&mut input, PublicKey::len())?;

                let len = util::read_and_advance_u64(&mut input)?;
                let input = input.len() >= len as _; //check msg len

                let out = out.len() >= Signature::len();

                if input && out {
                    Ok(())
                } else {
                    Err(Error::OutOfMemory)
                }
            }
        }
    }
}

impl<'r> TaApp<'r> {
    pub fn with_rng<R: CryptoRng + RngCore + 'r>(rng: &'r mut R) -> Self {
        Self {
            rng: rng as _,
            keys: Default::default(),
        }
    }

    fn find_associated_key(&self, public_key: PublicKey) -> Option<SecretKey> {
        let mut keys = self.keys.iter();
        while let Some(Some(pair)) = keys.next() {
            if pair.public == public_key {
                return Some(pair.secret.clone()); //this is just 64 bytes
            }
        }

        None
    }

    /// Sign a message with the given secret key (and public key)
    fn sign(&mut self, sk: &SecretKey, ctx: &[u8], msg: &[u8], pk: &PublicKey) -> Signature {
        util::sign::sign_with_rng(&mut self.rng, sk, ctx, msg, pk)
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
    use std::{boxed::Box, dbg, vec, vec::Vec};

    use super::*;
    use optee_common::Serialize;

    impl Default for TaApp<'static> {
        fn default() -> Self {
            let rng = Box::new(rand::thread_rng());

            Self {
                rng: Box::leak(rng),
                keys: Default::default(),
            }
        }
    }

    impl<'r> TaApp<'r> {
        fn set_keys(&mut self, keypairs: &[&Keypair]) {
            let keys: Vec<_> = keypairs
                .iter()
                .take(self.keys.len())
                .map(|k| Some((*k).clone()))
                .collect();

            self.keys.clone_from_slice(keys.as_slice());
        }
    }

    fn keypair() -> Keypair {
        Keypair::generate_with(&mut rand::thread_rng())
    }

    #[test]
    fn get_random_key() {
        let mut rng = rand_core::OsRng;
        let mut app = TaApp::with_rng(&mut rng);

        let input = "".serialize().unwrap();

        let mut output = Vec::new();
        output.resize(PublicKey::len(), 0);

        app.process_command(CommandId::GenerateNew, &input[..], &mut output)
            .expect("shouldn't fail");

        let valid_pk = PublicKey::from_bytes(&output).expect("not a valid public key");
        dbg!(valid_pk);
    }

    #[test]
    fn sign_something() {
        let mut rng = rand_core::OsRng;
        let mut app = TaApp::with_rng(&mut rng);

        let sk = keypair();
        app.set_keys(&[&sk]);

        let msg = &b"francesco@zondax.ch"[..];

        let input = {
            let mut vec = vec![0u8; PublicKey::len()];
            sk.public.serialize_fixed(&mut vec).unwrap();
            vec.append(&mut msg.serialize().unwrap());
            vec
        };

        let mut output = vec![0u8; Signature::len()];

        app.process_command(CommandId::SignMessage, &input[..], &mut output)
            .expect("shouldn't fail");

        let signature = Signature::from_bytes(&output).expect("not a valid signature key");
        dbg!(signature);

        let transcript = util::sign::get_transcript(&mut rng, b"zondax", msg);

        sk.public
            .verify(transcript, &signature)
            .expect("signature couldn't be verified");
    }
}
