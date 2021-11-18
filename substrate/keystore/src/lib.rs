use parking_lot::RwLock;
use sp_core::{
    crypto::{CryptoTypePublicPair, KeyTypeId},
    ecdsa, ed25519, sr25519,
};
use sp_keystore::Error;
use tokio::runtime::Handle;
use url::Url;
use zkms_ductile::{RemoteKeystore, RemoteKeystoreResponse};

#[macro_use]
extern crate tracing;

struct ZKMSClient {
    pub tx: ductile::ChannelSender<RemoteKeystore>,
    pub rx: ductile::ChannelReceiver<RemoteKeystoreResponse>,
}

/// A remote keystore
///
/// Talks to a zondax keystore via jsonrpc
pub struct TEEKeystore {
    client: RwLock<Option<ZKMSClient>>,

    url: Url,

    /// Handle to the tokio runtime
    runtime: Handle,
}

impl std::fmt::Debug for TEEKeystore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TEEKeystore")
    }
}

impl TEEKeystore {
    pub fn connect(&self) -> ductile::Result<()> {
        if self.client.read().as_ref().is_none() {
            debug!("creating new connection to TEE keystore");
            let host = self.url.host_str().expect("Invalid ip address");
            let port = self.url.port().expect("Invalid valid port");
            let (tx, rx) = ductile::connect_channel(format!("{}:{}", host, port))?;

            let handle = ZKMSClient { tx, rx };
            self.client.write().replace(handle);
            Ok(())
        } else {
            Ok(())
        }
    }

    #[instrument]
    pub fn deferred(url: &str) -> Result<Self, String> {
        let url: Url = url
            .parse()
            .map_err(|e| format!("cannot parse url: {:?}", e))?;
        if url.scheme() != "tcp" {
            return Err(format!(
                "Invalid scheme {} - only tcp is supported",
                url.scheme()
            ));
        }

        debug!("retrieving tokio runtime handle");
        let runtime = Handle::current();

        Ok(Self {
            client: RwLock::default(),
            url,
            runtime,
        })
    }
}

impl TEEKeystore {
    fn client(&self) -> parking_lot::RwLockReadGuard<'_, Option<ZKMSClient>> {
        let _ = self.connect();
        self.client.read()
    }

    #[instrument]
    fn sr25519_public_keys(&self, id: KeyTypeId) -> Vec<sr25519::Public> {
        let client = self.client();
        let client = match client.as_ref() {
            Some(client) => client,
            None => return vec![],
        };

        match client.tx.send(RemoteKeystore::Sr25519PublicKeys(id)) {
            Err(_) => vec![],
            Ok(_) => client
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

    #[instrument]
    fn sr25519_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<sr25519::Public, Error> {
        let client = self.client();
        let client = client.as_ref().ok_or(Error::Unavailable)?;

        match client.tx.send(RemoteKeystore::Sr25519GenerateNew {
            id,
            seed: seed.map(|s| s.to_string()),
        }) {
            Err(_) => Err(Error::Unavailable),
            Ok(_) => client
                .rx
                .recv()
                .map_err(|_| Error::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::Sr25519GenerateNew(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(Error::Unavailable)
                    }
                }),
        }
    }

    #[instrument]
    fn ed25519_public_keys(&self, id: KeyTypeId) -> Vec<ed25519::Public> {
        let client = self.client();
        let client = match client.as_ref() {
            Some(client) => client,
            None => return vec![],
        };

        match client.tx.send(RemoteKeystore::Ed25519PublicKeys(id)) {
            Err(_) => vec![],
            Ok(_) => client
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

    #[instrument]
    fn ed25519_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ed25519::Public, Error> {
        let client = self.client();
        let client = client.as_ref().ok_or(Error::Unavailable)?;

        match client.tx.send(RemoteKeystore::Ed25519GenerateNew {
            id,
            seed: seed.map(|s| s.to_string()),
        }) {
            Err(_) => Err(Error::Unavailable),
            Ok(_) => client
                .rx
                .recv()
                .map_err(|_| Error::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::Ed25519GenerateNew(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(Error::Unavailable)
                    }
                }),
        }
    }

    #[instrument]
    fn ecdsa_public_keys(&self, id: KeyTypeId) -> Vec<ecdsa::Public> {
        let client = self.client();
        let client = match client.as_ref() {
            Some(client) => client,
            None => return vec![],
        };

        match client.tx.send(RemoteKeystore::EcdsaPublicKeys(id)) {
            Err(_) => vec![],
            Ok(_) => client
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

    #[instrument]
    fn ecdsa_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ecdsa::Public, Error> {
        let client = self.client();
        let client = client.as_ref().ok_or(Error::Unavailable)?;

        match client.tx.send(RemoteKeystore::EcdsaGenerateNew {
            id,
            seed: seed.map(|s| s.to_string()),
        }) {
            Err(_) => Err(Error::Unavailable),
            Ok(_) => client
                .rx
                .recv()
                .map_err(|_| Error::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::EcdsaGenerateNew(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(Error::Unavailable)
                    }
                }),
        }
    }

    #[instrument]
    fn insert_unknown(&self, key_type: KeyTypeId, suri: &str, public: &[u8]) -> Result<(), ()> {
        let client = self.client();
        let client = client.as_ref().ok_or(())?;

        match client.tx.send(RemoteKeystore::InsertUnknown {
            id: key_type,
            suri: suri.to_string(),
            public: Vec::from(public),
        }) {
            Err(_) => Err(()),
            Ok(_) => client.rx.recv().map_err(|_| ()).and_then(|resp| {
                if let RemoteKeystoreResponse::InsertUnknown(resp) = resp {
                    resp
                } else {
                    //unreachable!()
                    Err(())
                }
            }),
        }
    }

    #[instrument]
    fn supported_keys(
        &self,
        id: KeyTypeId,
        keys: Vec<CryptoTypePublicPair>,
    ) -> Result<Vec<CryptoTypePublicPair>, Error> {
        let client = self.client();
        let client = client.as_ref().ok_or(Error::Unavailable)?;

        match client.tx.send(RemoteKeystore::SupportedKeys { id, keys }) {
            Err(_) => Err(Error::Unavailable),
            Ok(_) => client
                .rx
                .recv()
                .map_err(|_| Error::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::SupportedKeys(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(Error::Unavailable)
                    }
                }),
        }
    }

    #[instrument]
    fn keys(&self, id: KeyTypeId) -> Result<Vec<CryptoTypePublicPair>, Error> {
        let client = self.client();
        let client = client.as_ref().ok_or(Error::Unavailable)?;

        match client.tx.send(RemoteKeystore::Keys(id)) {
            Err(_) => Err(Error::Unavailable),
            Ok(_) => client
                .rx
                .recv()
                .map_err(|_| Error::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::Keys(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(Error::Unavailable)
                    }
                }),
        }
    }

    #[instrument]
    fn has_keys(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool {
        let client = self.client();
        let client = match client.as_ref() {
            Some(c) => c,
            None => return false,
        };

        match client
            .tx
            .send(RemoteKeystore::HasKeys(public_keys.to_vec()))
        {
            Err(_) => false,
            Ok(_) => client
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

    #[instrument]
    fn sign_with(
        &self,
        id: KeyTypeId,
        key: &CryptoTypePublicPair,
        msg: &[u8],
    ) -> Result<Vec<u8>, Error> {
        let client = self.client();
        let client = client.as_ref().ok_or(Error::Unavailable)?;

        match client.tx.send(RemoteKeystore::SignWith {
            id,
            key: key.clone(),
            msg: msg.to_vec(),
        }) {
            Err(_) => Err(Error::Unavailable),
            Ok(_) => client
                .rx
                .recv()
                .map_err(|_| Error::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::SignWith(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(Error::Unavailable)
                    }
                }),
        }
    }

    #[instrument]
    fn sign_with_any(
        &self,
        id: KeyTypeId,
        keys: Vec<CryptoTypePublicPair>,
        msg: &[u8],
    ) -> Result<(CryptoTypePublicPair, Vec<u8>), Error> {
        let client = self.client();
        let client = client.as_ref().ok_or(Error::Unavailable)?;

        match client.tx.send(RemoteKeystore::SignWithAny {
            id,
            keys,
            msg: msg.to_vec(),
        }) {
            Err(_) => Err(Error::Unavailable),
            Ok(_) => client
                .rx
                .recv()
                .map_err(|_| Error::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::SignWithAny(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(Error::Unavailable)
                    }
                }),
        }
    }

    #[instrument]
    fn sign_with_all(
        &self,
        id: KeyTypeId,
        keys: Vec<CryptoTypePublicPair>,
        msg: &[u8],
    ) -> Result<Vec<Result<Vec<u8>, Error>>, ()> {
        let client = self.client();
        let client = client.as_ref().ok_or(())?;

        match client.tx.send(RemoteKeystore::SignWithAll {
            id,
            keys,
            msg: msg.to_vec(),
        }) {
            Err(_) => Err(()),
            Ok(_) => client.rx.recv().map_err(|_| ()).and_then(|resp| {
                if let RemoteKeystoreResponse::SignWithAll(resp) = resp {
                    resp
                } else {
                    //unreachable!()
                    Err(())
                }
            }),
        }
    }

    #[instrument]
    fn sr25519_vrf_sign(
        &self,
        key_type: KeyTypeId,
        public: &sr25519::Public,
        transcript_data: sp_keystore::vrf::VRFTranscriptData,
    ) -> Result<sp_keystore::vrf::VRFSignature, Error> {
        let client = self.client();
        let client = client.as_ref().ok_or(Error::Unavailable)?;

        match client.tx.send(RemoteKeystore::Sr25519VrfSign {
            key_type,
            public: *public,
            transcript_data,
        }) {
            Err(_) => Err(Error::Unavailable),
            Ok(_) => client
                .rx
                .recv()
                .map_err(|_| Error::Unavailable)
                .and_then(|resp| {
                    if let RemoteKeystoreResponse::Sr25519VrfSign(resp) = resp {
                        resp
                    } else {
                        //unreachable!()
                        Err(Error::Unavailable)
                    }
                }),
        }
    }
}

mod cryptostore;
mod synccryptostore;
