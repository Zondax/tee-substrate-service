use super::*;

use std::convert::Infallible;

impl Serialize for &[u8] {
    type Error = Infallible;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let len = self.len();

        let mut vec = vec![0; 8 + len];
        vec[..8].copy_from_slice(&len.to_le_bytes()[..]);
        vec[8..].copy_from_slice(self);

        Ok(vec)
    }
}

impl Serialize for &str {
    type Error = Infallible;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        self.as_bytes().serialize()
    }
}
