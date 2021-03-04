use super::*;

use std::convert::Infallible;

impl<'de> Deserialize<'de> for &'de [u8] {
    type Error = Infallible;

    fn deserialize(mut input: &'de [u8]) -> Result<Self, Self::Error> {
        let len = {
            let mut array = [0; 8];
            array.copy_from_slice(&input[..8]);
            input = &input[8..];
            u64::from_le_bytes(array)
        };

        Ok(&input[..len as usize])
    }
}

impl<'de> Deserialize<'de> for &'de str {
    type Error = std::str::Utf8Error;

    fn deserialize(input: &'de [u8]) -> Result<Self, Self::Error> {
        let input: &[u8] = Deserialize::deserialize(input).unwrap();

        std::str::from_utf8(input)
    }
}
