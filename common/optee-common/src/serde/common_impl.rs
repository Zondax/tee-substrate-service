use super::*;

use std::convert::Infallible;

use crate::CryptoAlgo;

impl SerializeFixed for CryptoAlgo {
    type ErrorFixed = Infallible;

    fn len() -> usize {
        1
    }

    fn serialize_fixed(&self, dest: &mut [u8]) -> Result<(), Self::ErrorFixed> {
        let byte = (*self).into();

        dest.copy_from_slice(&[byte]);
        Ok(())
    }
}

impl DeserializeOwned for CryptoAlgo {
    type ErrorOwned = ();

    fn deserialize_owned(input: &[u8]) -> Result<Self, Self::ErrorOwned> {
        match input.get(0).ok_or(())? {
            0 => Ok(CryptoAlgo::Sr25519),
            1 => Ok(CryptoAlgo::Ed25519),
            2 => Ok(CryptoAlgo::Ecdsa),
            _ => Err(()),
        }
    }
}
