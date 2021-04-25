#![no_std]

extern crate no_std_compat as std;
use std::prelude::v1::*;

use std::cell::{Ref, RefCell, RefMut};

use optee_common::{
    CommandId, CryptoAlgo, Deserialize, DeserializeOwned, DeserializeVariable, HandleTaCommand,
    HasKeysPair, SerializeFixed, TeeErrorCode as Error,
};
use rand_core::{CryptoRng, RngCore};

use hashbrown::HashMap;

mod util;
use util::CSPRNG;

mod crypto;
use crypto::Keypair;

#[macro_use]
extern crate log;

pub struct TaApp<'r> {
    keys: HashMap<[u8; 4], Vec<Keypair>>,
    rng: &'r mut dyn CSPRNG, //the rng provider
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

        // Self::check_mem(cmd_id, input, &output)?;
        // trace!("checked mem succesfully...");

        match cmd_id {
            CommandId::GenerateNew => {
                let algo = CryptoAlgo::deserialize_owned(input).map_err(|_| Error::BadFormat)?;
                util::advance_slice(&mut input, CryptoAlgo::len()).unwrap();

                let key_type: [u8; 4] =
                    DeserializeOwned::deserialize_owned(input).map_err(|_| Error::BadFormat)?;

                //generate keypair
                let keypair = Keypair::generate_new(&mut self.rng, algo);

                let public = keypair.public();
                trace!("generated keypair; public = {:x?}", public);

                //copy into output
                if public.len() > output.len() {
                    return Err(Error::BadFormat);
                }
                output[..public.len()].copy_from_slice(public);

                //insert into own store
                self.keys.entry(key_type).or_default().push(keypair);

                Ok(())
            }
            CommandId::GetKeys => {
                todo!()
            }
            CommandId::SignMessage => {
                let algo = CryptoAlgo::deserialize_owned(input).map_err(|_| Error::BadFormat)?;
                util::advance_slice(&mut input, CryptoAlgo::len()).unwrap();

                //no need to keep going if the output buffer is already too small
                if algo.signature_len() > output.len() {
                    return Err(Error::BadFormat)?;
                }

                let key_type: [u8; 4] =
                    DeserializeOwned::deserialize_owned(input).map_err(|_| Error::BadFormat)?;
                util::advance_slice(&mut input, 4).unwrap();

                let public: &[u8] =
                    Deserialize::deserialize(input).map_err(|_| Error::BadFormat)?;
                util::advance_slice(&mut input, public.len()).unwrap();
                trace!("read public key: {:x?}", public);

                let msg: &[u8] = Deserialize::deserialize(&input).unwrap();
                trace!("read msg: {:x?}", msg);

                let pair = self
                    .find_associated_key(&key_type, public)
                    .ok_or(Error::BadParameters)?;
                trace!("got keypair");

                let sig = pair.sign(&mut self.rng, &msg);
                trace!("signed! sig={:x?}", sig);

                if sig.len() > output.len() {
                    //double check even if we checked at the start
                    return Err(Error::BadFormat);
                }
                output[..sig.len()].copy_from_slice(&sig);

                Ok(())
            }
            CommandId::HasKeys => {
                //check if we have 1 byte available for the bool output
                if output.len() < 1 {
                    return Err(Error::BadFormat);
                }

                let (_, pairs): (_, Vec<HasKeysPair>) =
                    DeserializeVariable::deserialize_variable(input)
                        .map_err(|_| Error::BadFormat)?;

                let search = pairs.into_iter().all(
                    |HasKeysPair {
                         key_type,
                         public_key,
                     }| {
                        self.find_associated_key(&key_type, public_key.as_slice())
                            .is_some()
                    },
                );

                if search {
                    output[0] = 1;
                } else {
                    output[0] = 0;
                }

                Ok(())
            }
            CommandId::VrfSign => {
                todo!()
            }
        }
    }
}

impl<'r> TaApp<'r> {
    // ///Makes sure the input and output slice have enough length
    // fn check_mem(cmd: CommandId, mut input: &[u8], mut out: &[u8]) -> Result<(), Error> {
    //     match cmd {
    //         CommandId::GenerateNew => {
    //             let len = util::read_and_advance_u64(&mut input)?;

    //             let input = input.len() >= len as _;
    //             let out = out.len() >= PublicKey::len();

    //             if input && out {
    //                 Ok(())
    //             } else {
    //                 Err(Error::OutOfMemory)
    //             }
    //         }
    //         CommandId::GetKeys => Ok(()),
    //         CommandId::SignMessage => {
    //             //we can skip the public key here

    //             //attempt to read public_key, error if failed
    //             let _ = util::read_and_advance(&mut input, PublicKey::len())?;

    //             let len = util::read_and_advance_u64(&mut input)?;
    //             let input = input.len() >= len as _; //check msg len

    //             let out = out.len() >= Signature::len();

    //             if input && out {
    //                 Ok(())
    //             } else {
    //                 Err(Error::OutOfMemory)
    //             }
    //         }
    //         CommandId::HasKeys => {
    //             todo!()
    //         }
    //         CommandId::VrfSign => {
    //             todo!()
    //         }
    //     }
    // }
}

impl<'r> TaApp<'r> {
    pub fn with_rng<R: CryptoRng + RngCore + 'r>(rng: &'r mut R) -> Self {
        Self {
            rng: rng as _,
            keys: Default::default(),
        }
    }

    fn find_associated_key(&self, key_type: &[u8; 4], public_key: &[u8]) -> Option<Keypair> {
        self.keys
            .get(key_type)
            .and_then(|keys| keys.iter().find(|k| k.public() == public_key))
            .cloned()
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
            let keys: Vec<_> = keypairs.iter().map(|k| (*k).clone()).collect();

            self.keys = keys;
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
