use std::prelude::v1::*;

use optee_common::TeeErrorCode as Error;
use rand_core::{CryptoRng, RngCore};

//used in TaApp to specify multiple traits for the trait object
pub trait CSPRNG: CryptoRng + RngCore {}
impl<R: CryptoRng + RngCore> CSPRNG for R {}

const U64_SIZE: usize = std::mem::size_of::<u64>();

/// Reads an u64 from the slice, advancing it
pub fn read_and_advance_u64(slice: &mut &[u8]) -> Result<u64, Error> {
    if slice.len() < U64_SIZE {
        return Err(Error::OutOfMemory);
    }

    //read and advance slice
    let mut tmp = [0; U64_SIZE];
    tmp.copy_from_slice(&slice[..U64_SIZE]);
    *slice = &slice[U64_SIZE..];

    Ok(u64::from_le_bytes(tmp))
}

/// Reads `amt` bytes and advance it
pub fn read_and_advance<'s>(slice: &mut &'s [u8], amt: usize) -> Result<&'s [u8], Error> {
    if slice.len() < amt {
        return Err(Error::OutOfMemory);
    }

    let out = &slice[..amt];
    trace!("read {} bytes: {:?}", amt, out);
    *slice = &slice[amt..];

    Ok(out)
}

/// Advance a slice by a fixed amount but do a bound check first
pub fn advance_slice(slice: &mut &[u8], amt: usize) -> Result<(), Error> {
    if slice.len() < amt {
        Err(Error::OutOfMemory)
    } else {
        *slice = &slice[amt..];
        Ok(())
    }
}

pub mod hasher {
    use core::hash::{BuildHasher, Hasher};

    #[derive(Default)]
    pub struct Builder;

    ///This hasher is not cryptographically secure or resistant to collisions
    /// the key for this hasher is only every supposed to be an [u8; 4]
    pub struct MyHasher(u32);

    impl BuildHasher for Builder {
        type Hasher = MyHasher;

        fn build_hasher(&self) -> Self::Hasher {
            MyHasher(0)
        }
    }

    impl Hasher for MyHasher {
        fn finish(&self) -> u64 {
            self.0 as u64
        }

        fn write(&mut self, bytes: &[u8]) {
            let mut inner = [0; 4];
            for (i, b) in bytes.iter().take(4).enumerate() {
                inner[i] = *b;
            }
            self.0 = u32::from_le_bytes(inner);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_u64() {
        let input = 0u64.to_le_bytes();

        let read = read_and_advance_u64(&mut &input[..]).expect("shouldn't error");

        assert_eq!(read, 0)
    }

    #[test]
    fn read_amt() {
        let input = vec![42; 42];

        let read = read_and_advance(&mut &input[..], 42).expect("shouldn't error");

        assert_eq!(&read[..], &input[..])
    }
}
