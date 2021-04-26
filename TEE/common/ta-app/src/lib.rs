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

        match cmd_id {
            CommandId::GenerateNew => {
                let algo = CryptoAlgo::deserialize_owned(input).map_err(|_| Error::BadFormat)?;
                util::advance_slice(&mut input, CryptoAlgo::len()).unwrap();
                trace!("GenerateNew: read algo: {:?}", algo);

                let key_type: [u8; 4] =
                    DeserializeOwned::deserialize_owned(input).map_err(|_| Error::BadFormat)?;
                trace!("GenerateNew: read key_type: {:x?}", key_type);

                //generate keypair
                let keypair = Keypair::generate_new(&mut self.rng, algo);

                let public = keypair.public();
                trace!("generated keypair; public = {:x?}", public);

                //copy into output
                if public.len() > output.len() {
                    return Err(Error::BadFormat);
                }
                output[..public.len()].copy_from_slice(public);
                trace!("written public key");

                //insert into own store
                self.keys.entry(key_type).or_default().push(keypair);
                trace!("inserted keypair into own store");

                Ok(())
            }
            CommandId::GetKeys => {
                //check space for n of keys
                if output.len() < 8 {
                    return Err(Error::BadFormat);
                }

                let algo = CryptoAlgo::deserialize_owned(input).map_err(|_| Error::BadFormat)?;
                util::advance_slice(&mut input, CryptoAlgo::len()).unwrap();
                trace!("read algo: {:?}", algo);

                let key_type: [u8; 4] =
                    DeserializeOwned::deserialize_owned(input).map_err(|_| Error::BadFormat)?;
                trace!("read key_type: {:x?}", key_type);

                //search for key_type associated keypairs of the given curve
                let keys = self
                    .keys
                    .entry(key_type)
                    .or_default()
                    .iter()
                    .filter(|keypair| keypair == &&algo)
                    .map(|keypair| keypair.public()); //get the public part of the key

                let mut n_keys_written = 0;
                for key in keys {
                    let written_size = 8 + n_keys_written as usize * algo.pubkey_len();

                    if output.len() < written_size {
                        return Err(Error::BadFormat); //no more space in output
                    }

                    output[written_size..written_size + algo.pubkey_len()].copy_from_slice(key);
                    trace!("written n={}; key={:x?}", n_keys_written, key);
                    n_keys_written += 1;
                }

                let n_items = n_keys_written as u64;
                output[..8].copy_from_slice(&n_items.to_le_bytes()[..]);
                trace!("written items n={}", n_items);

                Ok(())
            }
            CommandId::SignMessage => {
                let algo = CryptoAlgo::deserialize_owned(input).map_err(|_| Error::BadFormat)?;
                util::advance_slice(&mut input, CryptoAlgo::len()).unwrap();
                trace!("SignMessage: read algo: {:?}", algo);

                //no need to keep going if the output buffer is already too small
                if algo.signature_len() > output.len() {
                    return Err(Error::BadFormat)?;
                }

                let key_type: [u8; 4] =
                    DeserializeOwned::deserialize_owned(input).map_err(|_| Error::BadFormat)?;
                trace!("SignMessage: read key_type: {:x?}", key_type);
                util::advance_slice(&mut input, 4).unwrap();

                let public: &[u8] =
                    Deserialize::deserialize(input).map_err(|_| Error::BadFormat)?;
                util::advance_slice(&mut input, 8 + public.len()).unwrap();
                trace!("read public key: {:x?}", public);

                let msg: &[u8] = Deserialize::deserialize(input).map_err(|_| Error::BadFormat)?;
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
                trace!("signature written");

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
                trace!("read HasKeysPair: {:?}", pairs);

                let search = pairs.into_iter().all(
                    |HasKeysPair {
                         key_type,
                         public_key,
                     }| {
                        self.find_associated_key(&key_type, public_key.as_slice())
                            .is_some()
                    },
                );
                trace!("searched all keys; search={}", search);

                if search {
                    output[0] = 1;
                } else {
                    output[0] = 0;
                }

                Ok(())
            }
            CommandId::VrfSign => {
                let key_type: [u8; 4] =
                    DeserializeOwned::deserialize_owned(input).map_err(|_| Error::BadFormat)?;
                util::advance_slice(&mut input, 4).unwrap();
                trace!("read key_type: {:x?}", key_type);

                let public: &[u8] =
                    Deserialize::deserialize(input).map_err(|_| Error::BadFormat)?;
                util::advance_slice(&mut input, 8 + public.len()).unwrap();
                trace!("got public key={:x?}", public);

                let pair = self
                    .find_associated_key(&key_type, public)
                    .ok_or(Error::BadParameters)?;
                trace!("found keypair");

                let data: crypto::VRFData =
                    Deserialize::deserialize(input).map_err(|_| Error::BadFormat)?;
                trace!("got vrf data = {:?}", data);

                let vrf = pair
                    .vrf_sign(&mut self.rng, data)
                    .map_err(|_| Error::BadParameters)?;
                trace!("signed vrf={:x?}", vrf);

                if vrf.len() > output.len() {
                    return Err(Error::BadFormat);
                }
                output[..vrf.len()].copy_from_slice(&vrf);
                trace!("written vrf");

                Ok(())
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
    use crypto::PublicKey;
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

            let map: HashMap<[u8; 4], Vec<Keypair>> =
                [(*b"dumm", keys)].into_iter().cloned().collect();
            self.keys = map;
        }
    }

    fn keypair(algo: CryptoAlgo) -> Keypair {
        Keypair::generate_new(&mut rand::thread_rng(), algo)
    }

    fn init_logging() {
        env_logger::try_init();
    }

    const KEY_TYPE: [u8; 4] = *b"dumm";

    fn get_random_key(algo: CryptoAlgo) {
        let mut rng = rand_core::OsRng;
        let mut app = TaApp::with_rng(&mut rng);

        let mut input = algo.serialize().unwrap();
        input.append(&mut KEY_TYPE.serialize().unwrap());

        let mut output = Vec::new();
        output.resize(algo.pubkey_len(), 0);

        app.process_command(CommandId::GenerateNew, &input[..], &mut output)
            .expect("shouldn't fail");

        let key = PublicKey::from_bytes(algo, &output).expect("not a valid public key");
        dbg!(key);
    }

    #[test]
    fn get_random_keys() {
        init_logging();
        get_random_key(CryptoAlgo::Sr25519);
        get_random_key(CryptoAlgo::Ed25519);
        get_random_key(CryptoAlgo::Ecdsa);
    }

    fn sign_something(algo: CryptoAlgo) {
        let mut rng = rand_core::OsRng;
        let mut app = TaApp::with_rng(&mut rng);

        let sk = keypair(algo);
        trace!("genned keypair with public={:x?}", sk.public());
        app.set_keys(&[&sk]);

        let msg = &b"francesco@zondax.ch"[..];

        let mut input = algo.serialize().unwrap();
        input.append(&mut KEY_TYPE.serialize().unwrap());
        input.append(&mut (&sk.public()).serialize().unwrap());
        input.append(&mut (&msg).serialize().unwrap());

        let mut output = vec![0u8; algo.signature_len()];

        app.process_command(CommandId::SignMessage, &input[..], &mut output)
            .expect("shouldn't fail");

        let public: PublicKey = sk.into();
        assert!(public.verify(&mut rng, msg, &output));
    }

    #[test]
    fn verify_sign() {
        init_logging();
        sign_something(CryptoAlgo::Sr25519);
        sign_something(CryptoAlgo::Ed25519);
        sign_something(CryptoAlgo::Ecdsa);
    }
}
