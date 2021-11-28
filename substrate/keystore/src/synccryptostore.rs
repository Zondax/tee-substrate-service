use super::*;

impl sp_keystore::SyncCryptoStore for TEEKeystore {
    fn sr25519_public_keys(&self, id: KeyTypeId) -> Vec<sr25519::Public> {
        self.sr25519_public_keys(id)
    }

    fn sr25519_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<sr25519::Public, sp_keystore::Error> {
        self.sr25519_generate_new(id, seed)
    }

    fn ed25519_public_keys(&self, id: KeyTypeId) -> Vec<ed25519::Public> {
        self.ed25519_public_keys(id)
    }

    fn ed25519_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ed25519::Public, sp_keystore::Error> {
        self.ed25519_generate_new(id, seed)
    }

    fn ecdsa_public_keys(&self, id: KeyTypeId) -> Vec<ecdsa::Public> {
        self.ecdsa_public_keys(id)
    }

    fn ecdsa_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ecdsa::Public, sp_keystore::Error> {
        self.ecdsa_generate_new(id, seed)
    }

    fn insert_unknown(&self, id: KeyTypeId, suri: &str, public: &[u8]) -> Result<(), ()> {
        self.insert_unknown(id, suri, public)
    }

    fn supported_keys(
        &self,
        id: KeyTypeId,
        keys: Vec<CryptoTypePublicPair>,
    ) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
        self.supported_keys(id, keys)
    }

    fn keys(&self, id: KeyTypeId) -> Result<Vec<CryptoTypePublicPair>, sp_keystore::Error> {
        self.keys(id)
    }

    fn has_keys(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool {
        self.has_keys(public_keys)
    }

    fn sign_with(
        &self,
        id: KeyTypeId,
        key: &CryptoTypePublicPair,
        msg: &[u8],
    ) -> Result<Vec<u8>, sp_keystore::Error> {
        self.sign_with(id, key, msg)
    }

    fn sr25519_vrf_sign(
        &self,
        key_type: KeyTypeId,
        public: &sr25519::Public,
        transcript_data: sp_keystore::vrf::VRFTranscriptData,
    ) -> Result<sp_keystore::vrf::VRFSignature, sp_keystore::Error> {
        self.sr25519_vrf_sign(key_type, public, transcript_data)
        // Err(Error::Unavailable)
    }
}
