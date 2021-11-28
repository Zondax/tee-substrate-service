use super::*;

use schnorrkel::{
    vrf::{VRFPreOut, VRFProof},
    PublicKey, Signature, SignatureError, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH,
};
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

impl SerializeFixed for VRFPreOut {
    type ErrorFixed = usize;

    fn len() -> usize {
        32
    }

    fn serialize_fixed(&self, dest: &mut [u8]) -> Result<(), Self::ErrorFixed> {
        if dest.len() < Self::len() {
            return Err(Self::len());
        }

        dest[..Self::len()].copy_from_slice(&self.to_bytes()[..]);

        Ok(())
    }
}

impl DeserializeOwned for VRFPreOut {
    type ErrorOwned = usize;

    fn deserialize_owned(input: &[u8]) -> Result<Self, Self::ErrorOwned> {
        if input.len() < Self::len() {
            return Err(Self::len());
        }

        Self::from_bytes(&input[..Self::len()]).map_err(|_| Self::len())
    }
}

impl SerializeFixed for VRFProof {
    type ErrorFixed = usize;

    fn len() -> usize {
        64
    }

    fn serialize_fixed(&self, dest: &mut [u8]) -> Result<(), Self::ErrorFixed> {
        if dest.len() < Self::len() {
            return Err(Self::len());
        }

        dest[..Self::len()].copy_from_slice(&self.to_bytes()[..]);

        Ok(())
    }
}

impl DeserializeOwned for VRFProof {
    type ErrorOwned = SignatureError;

    fn deserialize_owned(input: &[u8]) -> Result<Self, Self::ErrorOwned> {
        Self::from_bytes(input) //this already checks for the length
    }
}
