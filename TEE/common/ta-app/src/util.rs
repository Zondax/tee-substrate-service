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
