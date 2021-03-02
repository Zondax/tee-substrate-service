use optee_common::TeeErrorCode as Error;
use rand_core::{CryptoRng, RngCore};

//used in TaApp to specify multiple traits for the trait object
pub trait CSPRNG: CryptoRng + RngCore {}
impl<R: CryptoRng + RngCore> CSPRNG for R {}

const U64_SIZE: usize = core::mem::size_of::<u64>();

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

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::vec;

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

pub mod sign {
    use super::*;
    use merlin::Transcript;
    use schnorrkel::{context::SigningTranscriptWithRng, PublicKey, SecretKey, Signature};

    pub fn get_transcript<R: CryptoRng + RngCore>(
        rng: R,
        ctx: &[u8],
        msg: &[u8],
    ) -> SigningTranscriptWithRng<Transcript, R> {
        let mut t = Transcript::new(b"ta_app::util::signing_transcript");
        t.append_message(b"", ctx);
        t.append_message(b"sign-bytes", msg);

        schnorrkel::context::attach_rng(t, rng)
    }

    pub fn sign_with_rng<R: CryptoRng + RngCore>(
        rng: R,
        sk: &SecretKey,
        ctx: &[u8],
        msg: &[u8],
        pk: &PublicKey,
    ) -> Signature {
        let t = get_transcript(rng, ctx, msg);

        sk.sign(t, pk)
    }
}
