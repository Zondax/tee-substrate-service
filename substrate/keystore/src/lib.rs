use sp_core::{
    crypto::{CryptoTypePublicPair, KeyTypeId},
    ecdsa, ed25519, sr25519,
};
use sp_keystore::CryptoStore;
use tokio::runtime::Handle;

/// A remote keystore
///
/// Talks to a zondax keystore via jsonrpc
pub struct TEEKeystore {
    handle: zkms_jsonrpc::ZKMSClient,
    /// Used for functionality not provided in ZKMSClient
    fallback: sc_keystore::LocalKeystore,

    /// Handle to the tokio runtime
    runtime: Handle,
}

impl TEEKeystore {
    pub async fn new(url: &str) -> jsonrpc_core_client::RpcResult<Self> {
        let handle = jsonrpc_core_client::transports::http::connect(url).await?;
        let fallback = sc_keystore::LocalKeystore::in_memory();

        let runtime = tokio::runtime::Handle::current();

        Ok(Self {
            handle,
            fallback,
            runtime,
        })
    }
}

#[cfg(feature = "self-tokio")]
static RUNTIME: once_cell::sync::Lazy<tokio::runtime::Runtime> =
    once_cell::sync::Lazy::new(|| tokio::runtime::Builder::new().enable_all().build().unwrap());

#[cfg(feature = "self-tokio")]
impl TEEKeystore {
    pub fn new_sync(url: &str) -> jsonrpc_core_client::RpcResult<Self> {
        let handle = tokio::runtime::Handle::try_current();
        (&handle.as_ref())
            .unwrap_or_else(|_| RUNTIME.handle())
            .block_on(Self::new(url))
    }
}

impl TEEKeystore {
    async fn sr25519_public_keys_impl(&self, _: KeyTypeId) -> Vec<sr25519::Public> {
        let keys = self.handle.get_public_keys().await.unwrap_or_default();
        keys.into_iter()
            .map(|k| sr25519::Public::from_raw(k))
            .collect()
    }

    async fn sr25519_generate_new_impl(
        &self,
        _: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<sr25519::Public, sp_keystore::Error> {
        self.handle
            .generate_new(seed.map(|s| s.to_string()))
            .await
            .map_err(|_| sp_keystore::Error::Unavailable)
            .map(|k| sr25519::Public::from_raw(k))
    }

    async fn ed25519_public_keys_impl(&self, id: KeyTypeId) -> Vec<ed25519::Public> {
        self.fallback.ed25519_public_keys(id).await
    }

    async fn ed25519_generate_new_impl(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ed25519::Public, sp_keystore::Error> {
        self.fallback.ed25519_generate_new(id, seed).await
    }

    async fn ecdsa_public_keys_impl(&self, id: KeyTypeId) -> Vec<ecdsa::Public> {
        self.fallback.ecdsa_public_keys(id).await
    }

    async fn ecdsa_generate_new_impl(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ecdsa::Public, sp_keystore::Error> {
        self.ecdsa_generate_new(id, seed).await
    }

    async fn insert_unknown_impl(
        &self,
        id: KeyTypeId,
        suri: &str,
        public: &[u8],
    ) -> Result<(), ()> {
        self.fallback.insert_unknown(id, suri, public).await
    }

    async fn supported_keys_impl(
        &self,
        id: KeyTypeId,
        keys: Vec<CryptoTypePublicPair>,
    ) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
        self.fallback.supported_keys(id, keys).await
    }

    async fn keys_impl(
        &self,
        id: KeyTypeId,
    ) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
        self.fallback.keys(id).await
    }

    async fn has_keys_impl(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool {
        self.fallback.has_keys(public_keys).await
    }

    async fn sign_with_impl(
        &self,
        _: KeyTypeId,
        key: &CryptoTypePublicPair,
        msg: &[u8],
    ) -> Result<Vec<u8>, sp_keystore::Error> {
        let key = {
            let mut array = [0u8; 32];
            array.copy_from_slice(&key.1[..32]);
            array
        };

        self.handle
            .sign_message(key, msg.to_vec())
            .await
            .map_err(|_| sp_keystore::Error::Unavailable)
    }

    async fn sr25519_vrf_sign_impl(
        &self,
        key_type: KeyTypeId,
        public: &sr25519::Public,
        transcript_data: sp_keystore::vrf::VRFTranscriptData,
    ) -> Result<sp_keystore::vrf::VRFSignature, sp_keystore::Error> {
        self.fallback
            .sr25519_vrf_sign(key_type, public, transcript_data)
            .await
    }
}

mod cryptostore {
    use super::*;
    use async_trait::async_trait;

    #[async_trait]
    impl CryptoStore for TEEKeystore {
        async fn sr25519_public_keys(&self, id: KeyTypeId) -> Vec<sr25519::Public> {
            self.sr25519_public_keys_impl(id).await
        }

        async fn sr25519_generate_new(
            &self,
            id: KeyTypeId,
            seed: Option<&str>,
        ) -> Result<sr25519::Public, sp_keystore::Error> {
            self.sr25519_generate_new_impl(id, seed).await
        }

        async fn ed25519_public_keys(&self, id: KeyTypeId) -> Vec<ed25519::Public> {
            self.ed25519_public_keys_impl(id).await
        }

        async fn ed25519_generate_new(
            &self,
            id: KeyTypeId,
            seed: Option<&str>,
        ) -> Result<ed25519::Public, sp_keystore::Error> {
            self.ed25519_generate_new_impl(id, seed).await
        }

        async fn ecdsa_public_keys(&self, id: KeyTypeId) -> Vec<ecdsa::Public> {
            self.ecdsa_public_keys_impl(id).await
        }

        async fn ecdsa_generate_new(
            &self,
            id: KeyTypeId,
            seed: Option<&str>,
        ) -> Result<ecdsa::Public, sp_keystore::Error> {
            self.ecdsa_generate_new_impl(id, seed).await
        }

        async fn insert_unknown(&self, id: KeyTypeId, suri: &str, public: &[u8]) -> Result<(), ()> {
            self.insert_unknown_impl(id, suri, public).await
        }

        async fn supported_keys(
            &self,
            id: KeyTypeId,
            keys: Vec<CryptoTypePublicPair>,
        ) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
            self.supported_keys_impl(id, keys).await
        }

        async fn keys(
            &self,
            id: KeyTypeId,
        ) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
            self.keys_impl(id).await
        }

        async fn has_keys(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool {
            self.has_keys_impl(public_keys).await
        }

        async fn sign_with(
            &self,
            id: KeyTypeId,
            key: &CryptoTypePublicPair,
            msg: &[u8],
        ) -> Result<Vec<u8>, sp_keystore::Error> {
            self.sign_with_impl(id, key, msg).await
        }

        async fn sr25519_vrf_sign(
            &self,
            key_type: KeyTypeId,
            public: &sr25519::Public,
            transcript_data: sp_keystore::vrf::VRFTranscriptData,
        ) -> Result<sp_keystore::vrf::VRFSignature, sp_keystore::Error> {
            self.sr25519_vrf_sign_impl(key_type, public, transcript_data)
                .await
        }
    }
}

mod synccryptostore {
    use super::*;
    use sp_keystore::SyncCryptoStore;

    impl SyncCryptoStore for TEEKeystore {
        fn sr25519_public_keys(&self, id: KeyTypeId) -> Vec<sr25519::Public> {
            self.runtime.block_on(self.sr25519_public_keys_impl(id))
        }

        fn sr25519_generate_new(
            &self,
            id: KeyTypeId,
            seed: Option<&str>,
        ) -> Result<sr25519::Public, sp_keystore::Error> {
            self.runtime
                .block_on(self.sr25519_generate_new_impl(id, seed))
        }

        fn ed25519_public_keys(&self, id: KeyTypeId) -> Vec<ed25519::Public> {
            self.runtime.block_on(self.ed25519_public_keys_impl(id))
        }

        fn ed25519_generate_new(
            &self,
            id: KeyTypeId,
            seed: Option<&str>,
        ) -> Result<ed25519::Public, sp_keystore::Error> {
            self.runtime
                .block_on(self.ed25519_generate_new_impl(id, seed))
        }

        fn ecdsa_public_keys(&self, id: KeyTypeId) -> Vec<ecdsa::Public> {
            self.runtime.block_on(self.ecdsa_public_keys_impl(id))
        }

        fn ecdsa_generate_new(
            &self,
            id: KeyTypeId,
            seed: Option<&str>,
        ) -> Result<ecdsa::Public, sp_keystore::Error> {
            self.runtime
                .block_on(self.ecdsa_generate_new_impl(id, seed))
        }

        fn insert_unknown(&self, key_type: KeyTypeId, suri: &str, public: &[u8]) -> Result<(), ()> {
            self.runtime
                .block_on(self.insert_unknown_impl(key_type, suri, public))
        }

        fn supported_keys(
            &self,
            id: KeyTypeId,
            keys: Vec<CryptoTypePublicPair>,
        ) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
            self.runtime.block_on(self.supported_keys_impl(id, keys))
        }

        fn has_keys(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool {
            self.runtime.block_on(self.has_keys_impl(public_keys))
        }

        fn sign_with(
            &self,
            id: KeyTypeId,
            key: &CryptoTypePublicPair,
            msg: &[u8],
        ) -> Result<Vec<u8>, sp_keystore::Error> {
            self.runtime.block_on(self.sign_with_impl(id, key, msg))
        }

        fn sr25519_vrf_sign(
            &self,
            key_type: KeyTypeId,
            public: &sr25519::Public,
            transcript_data: sp_keystore::vrf::VRFTranscriptData,
        ) -> Result<sp_keystore::vrf::VRFSignature, sp_keystore::Error> {
            self.runtime
                .block_on(self.sr25519_vrf_sign_impl(key_type, public, transcript_data))
        }
    }
}
