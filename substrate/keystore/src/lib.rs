use async_trait::async_trait;
use sp_core::{
    crypto::{CryptoTypePublicPair, KeyTypeId},
    ecdsa, ed25519, sr25519,
};
use sp_keystore::CryptoStore;

/// A remote keystore
///
/// Talks to a zondax keystore via jsonrpc
pub struct TEEKeystore {
    handle: zkms_jsonrpc::ZKMSClient,
    /// Used for functionality not provided in ZKMSClient
    fallback: sc_keystore::LocalKeystore,
}

impl TEEKeystore {
    pub async fn new(url: &str) -> jsonrpc_core_client::RpcResult<Self> {
        let handle = jsonrpc_core_client::transports::http::connect(url).await?;
        let fallback = sc_keystore::LocalKeystore::in_memory();

        Ok(Self { handle, fallback })
    }
}

#[async_trait]
impl CryptoStore for TEEKeystore {
    async fn sr25519_public_keys(&self, _: KeyTypeId) -> Vec<sr25519::Public> {
        let keys = self.handle.get_public_keys().await.unwrap_or_default();
        keys.into_iter()
            .map(|k| sr25519::Public::from_raw(k))
            .collect()
    }

    async fn sr25519_generate_new(
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

    async fn ed25519_public_keys(&self, id: KeyTypeId) -> Vec<ed25519::Public> {
        self.fallback.ed25519_public_keys(id).await
    }

    async fn ed25519_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ed25519::Public, sp_keystore::Error> {
        self.fallback.ed25519_generate_new(id, seed).await
    }

    async fn ecdsa_public_keys(&self, id: KeyTypeId) -> Vec<ecdsa::Public> {
        self.fallback.ecdsa_public_keys(id).await
    }

    async fn ecdsa_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ecdsa::Public, sp_keystore::Error> {
        self.ecdsa_generate_new(id, seed).await
    }

    async fn insert_unknown(&self, id: KeyTypeId, suri: &str, public: &[u8]) -> Result<(), ()> {
        self.fallback.insert_unknown(id, suri, public).await
    }

    async fn supported_keys(
        &self,
        id: KeyTypeId,
        keys: Vec<CryptoTypePublicPair>,
    ) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
        self.fallback.supported_keys(id, keys).await
    }

    async fn keys(&self, id: KeyTypeId) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
        self.fallback.keys(id).await
    }

    async fn has_keys(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool {
        self.fallback.has_keys(public_keys).await
    }

    async fn sign_with(
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

    async fn sr25519_vrf_sign(
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
