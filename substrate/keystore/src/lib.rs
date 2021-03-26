use sp_core::{
    crypto::{CryptoTypePublicPair, KeyTypeId},
    ecdsa, ed25519, sr25519,
};
use sp_keystore::SyncCryptoStore;
use tokio::runtime::Handle;

#[macro_use]
extern crate tracing;

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

impl std::fmt::Debug for TEEKeystore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TEEKeystore")
    }
}

impl TEEKeystore {
    #[instrument]
    pub async fn new(url: &str) -> jsonrpc_core_client::RpcResult<Self> {
        debug!("creating new connection to TEE keystore");
        let handle = jsonrpc_core_client::transports::http::connect(&url).await?;

        debug!("creating fallback keystore");
        let fallback = sc_keystore::LocalKeystore::in_memory();

        debug!("retrieving tokio runtime handle");
        let runtime = tokio::runtime::Handle::current();

        debug!("got hanlde, returning Self");
        Ok(Self {
            handle,
            fallback,
            runtime,
        })
    }
}

mod scope;
use scope::execute_fut;

#[cfg(feature = "self-tokio")]
static RUNTIME: once_cell::sync::Lazy<tokio::runtime::Runtime> =
    once_cell::sync::Lazy::new(|| tokio::runtime::Builder::new().enable_all().build().unwrap());

#[cfg(feature = "self-tokio")]
impl TEEKeystore {
    pub fn new_sync(url: &str) -> jsonrpc_core_client::RpcResult<Self> {
        let handle = tokio::runtime::Handle::try_current();
        let handle = (&handle.as_ref()).unwrap_or_else(|_| RUNTIME.handle());

        execute_fut(Self::new(url), handle)
    }
}

impl TEEKeystore {
    #[instrument]
    async fn sr25519_public_keys_impl(&self, _: KeyTypeId) -> Vec<sr25519::Public> {
        let keys = self.handle.get_public_keys().await.unwrap_or_default();
        keys.into_iter()
            .map(|k| sr25519::Public::from_raw(k))
            .collect()
    }

    #[instrument]
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

    #[instrument]
    fn ed25519_public_keys_impl(&self, id: KeyTypeId) -> Vec<ed25519::Public> {
        self.fallback.ed25519_public_keys(id)
    }

    #[instrument]
    fn ed25519_generate_new_impl(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ed25519::Public, sp_keystore::Error> {
        self.fallback.ed25519_generate_new(id, seed)
    }

    #[instrument]
    fn ecdsa_public_keys_impl(&self, id: KeyTypeId) -> Vec<ecdsa::Public> {
        self.fallback.ecdsa_public_keys(id)
    }

    #[instrument]
    fn ecdsa_generate_new_impl(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ecdsa::Public, sp_keystore::Error> {
        self.fallback.ecdsa_generate_new(id, seed)
    }

    #[instrument]
    fn insert_unknown_impl(
        &self,
        id: KeyTypeId,
        suri: &str,
        public: &[u8],
    ) -> Result<(), ()> {
        self.fallback.insert_unknown(id, suri, public)
    }

    #[instrument]
    fn supported_keys_impl(
        &self,
        id: KeyTypeId,
        keys: Vec<CryptoTypePublicPair>,
    ) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
        self.fallback.supported_keys(id, keys)
    }

    #[instrument]
    fn keys_impl(
        &self,
        id: KeyTypeId,
    ) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
        self.fallback.keys(id)
    }

    #[instrument]
    fn has_keys_impl(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool {
        self.fallback.has_keys(public_keys)
    }

    #[instrument]
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

    #[instrument(skip(transcript_data))]
    fn sr25519_vrf_sign_impl(
        &self,
        key_type: KeyTypeId,
        public: &sr25519::Public,
        transcript_data: sp_keystore::vrf::VRFTranscriptData,
    ) -> Result<sp_keystore::vrf::VRFSignature, sp_keystore::Error> {
        self.fallback
            .sr25519_vrf_sign(key_type, public, transcript_data)
    }
}

mod cryptostore;
mod synccryptostore;
