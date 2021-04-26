use crate::crypto::VRFData;

use super::*;
use crypto::PublicKey;
use merlin::Transcript;
use optee_common::Serialize;
use sp_keystore::vrf::{VRFSignature, VRFTranscriptData};

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

        let mut map = HashMap::new();
        map.insert(KEY_TYPE, keys);

        self.keys = map;
    }
}

fn keypair(algo: CryptoAlgo) -> Keypair {
    Keypair::generate_new(&mut rand::thread_rng(), algo)
}

fn init_logging() {
    let _ = env_logger::try_init();
}

const KEY_TYPE: [u8; 4] = *b"dumm";

fn generate_new(algo: CryptoAlgo) {
    let mut app = TaApp::default();

    let mut input = algo.serialize().unwrap();
    input.append(&mut KEY_TYPE.serialize().unwrap());

    let mut output = Vec::new();
    output.resize(algo.pubkey_len(), 0);

    app.process_command(CommandId::GenerateNew, &input[..], &mut output)
        .expect("shouldn't fail");

    PublicKey::from_bytes(algo, &output).expect("not a valid public key");
}

#[test]
fn verify_generate_new() {
    init_logging();
    generate_new(CryptoAlgo::Sr25519);
    generate_new(CryptoAlgo::Ed25519);
    generate_new(CryptoAlgo::Ecdsa);
}

fn sign_something(algo: CryptoAlgo) {
    let mut app = TaApp::default();

    let sk = keypair(algo);
    trace!("genned keypair with public={:x?}", sk.public_bytes());
    app.set_keys(&[&sk]);

    let msg = &b"francesco@zondax.ch"[..];

    let mut input = algo.serialize().unwrap();
    input.append(&mut KEY_TYPE.serialize().unwrap());
    input.append(&mut (&sk.public_bytes()).serialize().unwrap());
    input.append(&mut (&msg).serialize().unwrap());

    let mut output = vec![0u8; algo.signature_len()];

    app.process_command(CommandId::SignMessage, &input[..], &mut output)
        .expect("shouldn't fail");

    let public = sk.to_public_key();
    assert!(public.verify(&mut rand::thread_rng(), msg, &output));
}

#[test]
fn verify_sign() {
    init_logging();
    sign_something(CryptoAlgo::Sr25519);
    sign_something(CryptoAlgo::Ed25519);
    sign_something(CryptoAlgo::Ecdsa);
}

fn get_public_keys<const GEN_KEYS: usize, const OUT_KEYS: usize>(algo: CryptoAlgo) {
    let mut app = TaApp::default();

    //fill with keys
    let set = vec![keypair(algo); GEN_KEYS];
    app.set_keys(set.iter().collect::<Vec<_>>().as_slice());

    let mut input = algo.serialize().unwrap();
    input.append(&mut KEY_TYPE.serialize().unwrap());

    let mut output = vec![0u8; 8 + (8 + algo.pubkey_len()) * OUT_KEYS];

    let result = app.process_command(CommandId::GetKeys, &input[..], &mut output);

    if GEN_KEYS > OUT_KEYS {
        result.expect_err("not out of memory");
        return;
    } else {
        result.expect("shouldn't fail");
    }

    let keys: Vec<Vec<u8>> = DeserializeVariable::deserialize_variable(&output)
        .unwrap()
        .1;

    for key in keys.iter() {
        assert!(PublicKey::from_bytes(algo, key).is_ok());
    }
}

#[test]
fn verify_get_keys() {
    init_logging();
    get_public_keys::<1, 1>(CryptoAlgo::Sr25519);
    get_public_keys::<1, 1>(CryptoAlgo::Ed25519);
    get_public_keys::<1, 1>(CryptoAlgo::Ecdsa);
    get_public_keys::<5, 5>(CryptoAlgo::Sr25519);
    get_public_keys::<5, 5>(CryptoAlgo::Ed25519);
    get_public_keys::<5, 5>(CryptoAlgo::Ecdsa);
    get_public_keys::<3, 1>(CryptoAlgo::Sr25519);
    get_public_keys::<3, 1>(CryptoAlgo::Ed25519);
    get_public_keys::<3, 1>(CryptoAlgo::Ecdsa);
}

fn has_keys(algo: CryptoAlgo) {
    let mut app = TaApp::default();

    let sk = keypair(algo);
    trace!("genned keypair with public={:x?}", sk.public_bytes());
    app.set_keys(&[&sk]);

    let pairs = vec![HasKeysPair {
        key_type: KEY_TYPE,
        public_key: sk.public_bytes().to_vec(),
    }];
    let input = pairs.serialize().unwrap();
    trace!("pairs={:x?}, input={:x?}", pairs, input);

    let mut output = [0];

    app.process_command(CommandId::HasKeys, &input[..], &mut output)
        .expect("shouldn't fail");

    assert_eq!(output[0], 1);

    let pairs: Vec<HasKeysPair> = vec![];
    let input = pairs.serialize().unwrap();
    output[0] = 0;

    app.process_command(CommandId::HasKeys, &input[..], &mut output)
        .expect("shouldn't fail");

    assert_eq!(output[0], 1);

    let pairs = vec![HasKeysPair {
        key_type: KEY_TYPE,
        public_key: vec![42; 32],
    }];
    let input = pairs.serialize().unwrap();
    output[0] = 0;

    app.process_command(CommandId::HasKeys, &input[..], &mut output)
        .expect("shouldn't fail");

    assert_eq!(output[0], 0);
}

#[test]
fn verify_has_keys() {
    init_logging();
    has_keys(CryptoAlgo::Sr25519);
    has_keys(CryptoAlgo::Ed25519);
    has_keys(CryptoAlgo::Ecdsa);
}

fn get_vrf() -> (Transcript, VRFTranscriptData) {
    use sp_keystore::vrf::VRFTranscriptValue;
    use std::borrow::Cow;

    let mut t = Transcript::new(b"My label");
    t.append_u64(b"one", 1);
    t.append_message(b"two", "test".as_bytes());

    let vrf = VRFTranscriptData {
        label: Cow::from(&b"My label"[..]),
        items: vec![
            (Cow::from(&b"one"[..]), VRFTranscriptValue::U64(1)),
            (
                Cow::from(&b"two"[..]),
                VRFTranscriptValue::Bytes("test".as_bytes().to_vec()),
            ),
        ],
    };

    (t, vrf)
}

#[test]
#[ignore = "https://github.com/w3f/schnorrkel/issues/70"]
fn verify_vrf_sign() {
    init_logging();
    let mut app = TaApp::default();

    let sk = keypair(CryptoAlgo::Sr25519);
    trace!("genned keypair with public={:x?}", sk.public_bytes());
    app.set_keys(&[&sk]);

    let (t, vrf) = get_vrf();
    let serialized_vrf = vrf.serialize().unwrap();
    trace!("vrf={:x?}", serialized_vrf);

    let data = VRFData::deserialize(serialized_vrf.as_slice()).expect("can't deserialize VRFData");

    let sig = sk.vrf_sign(&mut rand::thread_rng(), data.clone());

    let mut input = KEY_TYPE.serialize().unwrap();
    input.append(&mut (&sk.public_bytes()).serialize().unwrap());
    input.extend_from_slice(serialized_vrf.as_slice());
    trace!("input = {:x?}", input);

    let mut output = vec![0; VRFSignature::len()];

    app.process_command(CommandId::VrfSign, &input[..], &mut output)
        .expect("shouldn't fail");

    let signature =
        VRFSignature::deserialize_owned(&output).expect("can't deserialize VRFSignature");

    let vrf_verify = sk
        .to_public_key()
        .vrf_verify(&mut rand::thread_rng(), data, &output);

    assert!(vrf_verify);
}
