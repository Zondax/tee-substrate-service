use super::*;
use async_trait::async_trait;

#[async_trait]
impl sp_keystore::CryptoStore for TEEKeystore {
    async fn sr25519_public_keys(&self, id: KeyTypeId) -> Vec<sr25519::Public> {
        self.sr25519_public_keys(id)
    }

    async fn sr25519_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<sr25519::Public, sp_keystore::Error> {
        self.sr25519_generate_new(id, seed)
    }

    async fn ed25519_public_keys(&self, id: KeyTypeId) -> Vec<ed25519::Public> {
        self.ed25519_public_keys(id)
    }

    async fn ed25519_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ed25519::Public, sp_keystore::Error> {
        self.ed25519_generate_new(id, seed)
    }

    async fn ecdsa_public_keys(&self, id: KeyTypeId) -> Vec<ecdsa::Public> {
        self.ecdsa_public_keys(id)
    }

    async fn ecdsa_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ecdsa::Public, sp_keystore::Error> {
        self.ecdsa_generate_new(id, seed)
    }

    async fn insert_unknown(&self, id: KeyTypeId, suri: &str, public: &[u8]) -> Result<(), ()> {
        self.insert_unknown(id, suri, public)
    }

    async fn supported_keys(
        &self,
        id: KeyTypeId,
        keys: Vec<CryptoTypePublicPair>,
    ) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
        self.supported_keys(id, keys)
    }

    async fn keys(&self, id: KeyTypeId) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
        self.keys(id)
    }

    async fn has_keys(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool {
        self.has_keys(public_keys)
    }

    async fn sign_with(
        &self,
        id: KeyTypeId,
        key: &CryptoTypePublicPair,
        msg: &[u8],
    ) -> Result<Vec<u8>, sp_keystore::Error> {
        self.sign_with(id, key, msg)
    }

    async fn sr25519_vrf_sign(
        &self,
        key_type: KeyTypeId,
        public: &sr25519::Public,
        transcript_data: sp_keystore::vrf::VRFTranscriptData,
    ) -> Result<sp_keystore::vrf::VRFSignature, sp_keystore::Error> {
        self.sr25519_vrf_sign(key_type, public, transcript_data)
        // Err(Error::Unavailable)
    }
}
