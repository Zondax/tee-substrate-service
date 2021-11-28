use std::borrow::Cow;
use std::net::ToSocketAddrs;

use ductile::{ChannelReceiver, ChannelSender};
use zkms_common::CryptoAlgo;
use zkms_ductile::{
    crypto::{self, KeyTypeId},
    ecdsa, ed25519, sr25519, KeystoreError, RemoteKeystore, RemoteKeystoreResponse, VRFSignature,
    VRFTranscriptData, VRFTranscriptValue,
};

pub struct Client {
    tx: ChannelSender<RemoteKeystore>,
    rx: ChannelReceiver<RemoteKeystoreResponse>,
}

const KEY_TYPE: [u8; 4] = *b"dumm";

impl Client {
    pub fn connect(addr: impl ToSocketAddrs) -> Option<Self> {
        let (tx, rx) = ductile::connect_channel(addr).ok()?;

        Some(Self { tx, rx })
    }

    pub fn sr25519_public_keys(&self) -> Vec<sr25519::Public> {
        match self
            .tx
            .send(RemoteKeystore::Sr25519PublicKeys(KeyTypeId(KEY_TYPE)))
        {
            Err(_) => vec![],
            Ok(_) => self
                .rx
                .recv()
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::Sr25519PublicKeys(resp) = resp {
                        Ok(resp)
                    } else {
                        //unreachable!()
                        Ok(vec![])
                    }
                })
                .or_else::<(), _>(|_| Ok(vec![]))
                .unwrap(),
        }
    }

    pub fn sr25519_generate_new(&self) -> Result<sr25519::Public, KeystoreError> {
        match self.tx.send(RemoteKeystore::Sr25519GenerateNew {
            id: KeyTypeId(KEY_TYPE),
            seed: None,
        }) {
            Err(_) => Err(KeystoreError::Unavailable),
            Ok(_) => self
                .rx
                .recv()
                .map_err(|_| KeystoreError::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::Sr25519GenerateNew(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(KeystoreError::Unavailable)
                    }
                }),
        }
    }

    pub fn ed25519_public_keys(&self) -> Vec<ed25519::Public> {
        match self
            .tx
            .send(RemoteKeystore::Ed25519PublicKeys(KeyTypeId(KEY_TYPE)))
        {
            Err(_) => vec![],
            Ok(_) => self
                .rx
                .recv()
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::Ed25519PublicKeys(resp) = resp {
                        Ok(resp)
                    } else {
                        //unreachable!()
                        Ok(vec![])
                    }
                })
                .or_else::<(), _>(|_| Ok(vec![]))
                .unwrap(),
        }
    }

    pub fn ed25519_generate_new(&self) -> Result<ed25519::Public, KeystoreError> {
        match self.tx.send(RemoteKeystore::Ed25519GenerateNew {
            id: KeyTypeId(KEY_TYPE),
            seed: None,
        }) {
            Err(_) => Err(KeystoreError::Unavailable),
            Ok(_) => self
                .rx
                .recv()
                .map_err(|_| KeystoreError::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::Ed25519GenerateNew(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(KeystoreError::Unavailable)
                    }
                }),
        }
    }

    pub fn ecdsa_public_keys(&self) -> Vec<ecdsa::Public> {
        match self
            .tx
            .send(RemoteKeystore::EcdsaPublicKeys(KeyTypeId(KEY_TYPE)))
        {
            Err(_) => vec![],
            Ok(_) => self
                .rx
                .recv()
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::EcdsaPublicKeys(resp) = resp {
                        Ok(resp)
                    } else {
                        //unreachable!()
                        Ok(vec![])
                    }
                })
                .or_else::<(), _>(|_| Ok(vec![]))
                .unwrap(),
        }
    }

    pub fn ecdsa_generate_new(&self) -> Result<ecdsa::Public, KeystoreError> {
        match self.tx.send(RemoteKeystore::EcdsaGenerateNew {
            id: KeyTypeId(KEY_TYPE),
            seed: None,
        }) {
            Err(_) => Err(KeystoreError::Unavailable),
            Ok(_) => self
                .rx
                .recv()
                .map_err(|_| KeystoreError::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::EcdsaGenerateNew(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(KeystoreError::Unavailable)
                    }
                }),
        }
    }

    pub fn has_keys(&self, public_keys: Vec<Vec<u8>>) -> bool {
        let pairs = public_keys
            .into_iter()
            .map(|v| (v, KeyTypeId(KEY_TYPE)))
            .collect();

        match self.tx.send(RemoteKeystore::HasKeys(pairs)) {
            Err(_) => false,
            Ok(_) => self
                .rx
                .recv()
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::HasKeys(resp) = resp {
                        Ok(resp)
                    } else {
                        //unreachable!()
                        Ok(false)
                    }
                })
                .or_else::<(), _>(|_| Ok(false))
                .unwrap(),
        }
    }

    pub fn sign_with(
        &self,
        algo: CryptoAlgo,
        key: Vec<u8>,
        msg: &[u8],
    ) -> Result<Vec<u8>, KeystoreError> {
        match self.tx.send(RemoteKeystore::SignWith {
            id: KeyTypeId(KEY_TYPE),
            key: crypto::CryptoTypePublicPair(crypto::CryptoTypeId(algo.into()), key),
            msg: msg.to_vec(),
        }) {
            Err(_) => Err(KeystoreError::Unavailable),
            Ok(_) => self
                .rx
                .recv()
                .map_err(|_| KeystoreError::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::SignWith(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(KeystoreError::Unavailable)
                    }
                }),
        }
    }

    pub fn sr25519_vrf_sign(
        &self,
        public: &sr25519::Public,
    ) -> Result<VRFSignature, KeystoreError> {
        let transcript_data = VRFTranscriptData {
            label: Cow::from(&b"My label"[..]),
            items: vec![
                (Cow::from(&b"one"[..]), VRFTranscriptValue::U64(1)),
                (
                    Cow::from(&b"two"[..]),
                    VRFTranscriptValue::Bytes("test".as_bytes().to_vec()),
                ),
            ],
        };

        match self.tx.send(RemoteKeystore::Sr25519VrfSign {
            key_type: KeyTypeId(KEY_TYPE),
            public: *public,
            transcript_data,
        }) {
            Err(_) => Err(KeystoreError::Unavailable),
            Ok(_) => self
                .rx
                .recv()
                .map_err(|_| KeystoreError::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::Sr25519VrfSign(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(KeystoreError::Unavailable)
                    }
                }),
        }
    }
}

/*
impl Client {
    pub fn sign_with_any(
        &self,
        keys: Vec<CryptoTypePublicPair>,
        msg: &[u8],
    ) -> Result<(CryptoTypePublicPair, Vec<u8>), KeystoreError> {
        match self.tx.send(RemoteKeystore::SignWithAny {
            id: KeyTypeId(KEY_TYPE),
            keys,
            msg: msg.to_vec(),
        }) {
            Err(_) => Err(KeystoreError::Unavailable),
            Ok(_) => self
                .rx
                .recv()
                .map_err(|_| KeystoreError::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::SignWithAny(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(KeystoreError::Unavailable)
                    }
                }),
        }
    }

    pub fn sign_with_all(
        &self,
        keys: Vec<CryptoTypePublicPair>,
        msg: &[u8],
    ) -> Result<Vec<Result<Vec<u8>, KeystoreError>>, ()> {
        match self.tx.send(RemoteKeystore::SignWithAll {
            id: KeyTypeId(KEY_TYPE),
            keys,
            msg: msg.to_vec(),
        }) {
            Err(_) => Err(()),
            Ok(_) => self.rx.recv().map_err(|_| ()).and_then(|resp| {
                if let RemoteKeystoreResponse::SignWithAll(resp) = resp {
                    resp
                } else {
                    //unreachable!()
                    Err(())
                }
            }),
        }
    }

    pub fn insert_unknown(&self, suri: &str, public: &[u8]) -> Result<(), ()> {
        match self.tx.send(RemoteKeystore::InsertUnknown {
            id: KeyTypeId(KEY_TYPE),
            suri: suri.to_string(),
            public: Vec::from(public),
        }) {
            Err(_) => Err(()),
            Ok(_) => self.rx.recv().map_err(|_| ()).and_then(|resp| {
                if let RemoteKeystoreResponse::InsertUnknown(resp) = resp {
                    resp
                } else {
                    //unreachable!()
                    Err(())
                }
            }),
        }
    }

    pub fn supported_keys(
        &self,
        keys: Vec<CryptoTypePublicPair>,
    ) -> Result<Vec<CryptoTypePublicPair>, KeystoreError> {
        match self.tx.send(RemoteKeystore::SupportedKeys {
            id: KeyTypeId(KEY_TYPE),
            keys,
        }) {
            Err(_) => Err(KeystoreError::Unavailable),
            Ok(_) => self
                .rx
                .recv()
                .map_err(|_| KeystoreError::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::SupportedKeys(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(KeystoreError::Unavailable)
                    }
                }),
        }
    }

    pub fn keys(&self) -> Result<Vec<CryptoTypePublicPair>, KeystoreError> {
        match self.tx.send(RemoteKeystore::Keys(KeyTypeId(KEY_TYPE))) {
            Err(_) => Err(KeystoreError::Unavailable),
            Ok(_) => self
                .rx
                .recv()
                .map_err(|_| KeystoreError::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::Keys(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(KeystoreError::Unavailable)
                    }
                }),
        }
    }
}
*/
