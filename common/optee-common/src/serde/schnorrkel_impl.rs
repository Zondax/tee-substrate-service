use super::*;

use schnorrkel::{PublicKey, Signature, SignatureError, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use std::convert::Infallible;

impl SerializeFixed for PublicKey {
    type ErrorFixed = Infallible;

    fn len() -> usize {
        PUBLIC_KEY_LENGTH
    }

    fn serialize_fixed(&self, dest: &mut [u8]) -> Result<(), Self::ErrorFixed> {
        let bytes = self.to_bytes();

        dest.copy_from_slice(&bytes[..]);
        Ok(())
    }
}

impl DeserializeOwned for PublicKey {
    type ErrorOwned = SignatureError;

    fn deserialize_owned(input: &[u8]) -> Result<Self, Self::ErrorOwned> {
        Self::from_bytes(&input[..Self::len()])
    }
}

impl SerializeFixed for Signature {
    type ErrorFixed = Infallible;

    fn len() -> usize {
        SIGNATURE_LENGTH
    }

    fn serialize_fixed(&self, dest: &mut [u8]) -> Result<(), Self::ErrorFixed> {
        let bytes = self.to_bytes();

        dest.copy_from_slice(&bytes[..]);
        Ok(())
    }
}

impl DeserializeOwned for Signature {
    type ErrorOwned = SignatureError;

    fn deserialize_owned(input: &[u8]) -> Result<Self, Self::ErrorOwned> {
        Self::from_bytes(&input[..Self::len()])
    }
}
