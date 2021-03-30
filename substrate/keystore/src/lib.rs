use sp_core::{
    crypto::{CryptoTypePublicPair, KeyTypeId},
    ecdsa, ed25519, sr25519,
};
use sp_keystore::SyncCryptoStore;
use tokio::runtime::Handle;
use tokio::sync::RwLock;

use zkms_jsonrpc::ZKMSClient;

#[macro_use]
extern crate tracing;

/// A remote keystore
///
/// Talks to a zondax keystore via jsonrpc
pub struct TEEKeystore {
    client: RwLock<Option<ZKMSClient>>,
    /// Used for functionality not provided in ZKMSClient
    fallback: sc_keystore::LocalKeystore,

    url: url::Url,

    /// Handle to the tokio runtime
    runtime: Handle,
}

impl std::fmt::Debug for TEEKeystore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TEEKeystore")
    }
}

impl TEEKeystore {
    pub async fn connect(&self) -> jsonrpc_core_client::RpcResult<()> {
        if self.client.read().await.as_ref().is_none() {
            debug!("creating new connection to TEE keystore");
            let handle =
                jsonrpc_core_client::transports::http::connect(self.url.to_string().as_str())
                    .await?;

            self.client.write().await.replace(handle);
            Ok(())
        } else {
            Ok(())
        }
    }

    #[instrument]
    pub fn deferred(url: &str) -> Result<Self, String> {
        let url = url
            .parse()
            .map_err(|e| format!("cannot parse url: {:?}", e))?;

        debug!("creating fallback keystore");
        let fallback = sc_keystore::LocalKeystore::in_memory();

        debug!("retrieving tokio runtime handle");
        let runtime = Handle::current();

        Ok(Self {
            client: RwLock::default(),
            fallback,
            url,
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

        let me = Self::deferred(url).map_err(jsonrpc_core_client::RpcError::Client)?;
        execute_fut(me.connect(), handle)?;
        Ok(me)
    }
}

impl TEEKeystore {
    async fn client(&self) -> tokio::sync::RwLockReadGuard<'_, ZKMSClient>{
        self.connect().await;
        let lock = self.client.read().await;
        tokio::sync::RwLockReadGuard::map(lock, |o| o.as_ref().unwrap())
    }

    #[instrument]
    async fn sr25519_public_keys_impl(&self, _: KeyTypeId) -> Vec<sr25519::Public> {
        let client = self.client().await;
        let keys = client.get_public_keys().await.unwrap_or_default();
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
        let client = self.client().await;
        client
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
    fn insert_unknown_impl(&self, id: KeyTypeId, suri: &str, public: &[u8]) -> Result<(), ()> {
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
    fn keys_impl(&self, id: KeyTypeId) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
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
        let client = self.client().await;

        let key = {
            let mut array = [0u8; 32];
            array.copy_from_slice(&key.1[..32]);
            array
        };

        client
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
