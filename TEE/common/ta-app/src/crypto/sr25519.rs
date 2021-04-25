use schnorrkel::{
    keys,
    vrf::{VRFPreOut, VRFProof},
};

use crate::util::CSPRNG;

#[derive(Clone)]
pub struct Keypair(keys::Keypair);

impl Keypair {
    pub fn generate<C: CSPRNG>(rng: &mut C) -> Self {
        Self(keys::Keypair::generate_with(rng))
    }

    pub fn public(&self) -> &[u8] {
        self.0.public.as_ref()
    }

    pub fn sign<C: CSPRNG>(&self, rng: &mut C, msg: &[u8]) -> [u8; 64] {
        let mut t = merlin::Transcript::new(b"SigningContext");
        t.append_message(b"", b"substrate"); //ctx
        t.append_message(b"sign-bytes", msg);

        let t = schnorrkel::context::attach_rng(t, rng);

        self.0.sign(t).to_bytes()
    }

    pub fn vrf_sign<C: CSPRNG>(&self, rng: &mut C, data: vrf::VRFData<'_>) -> crate::Vec<u8> {
        let t = vrf::make_transcript(data);
        let t = schnorrkel::context::attach_rng(t, rng);

        let (inout, proof, _) = self.0.vrf_sign(t);
        let preout = inout.to_preout();

        let preout = preout.to_bytes();
        let proof = proof.to_bytes();

        let midpoint = preout.len() as u64;
        let midpoint = midpoint.to_le_bytes();

        //VRFSignature serialized
        [&midpoint[..], &preout[..], &proof[..]].concat()

    }
}

mod vrf {
    use optee_common::{
        serde::{ArrayError, Tuple2Error},
        Deserialize, DeserializeOwned,
    };
    use merlin::Transcript;

    pub enum VRFValue<'de> {
        Bytes(&'de [u8]),
        U64(u64),
    }

    pub struct VRFData<'de> {
        label: &'de [u8],
        items: crate::Vec<(&'de [u8], VRFValue<'de>)>,
    }

    pub fn make_transcript(data: VRFData<'_>) -> Transcript {
        let mut transcript = Transcript::new(*StaticGuard::from_slice(data.label));

        for (label, value) in data.items.into_iter() {
            let label = StaticGuard::from_slice(label);
            match value {
                VRFValue::Bytes(bytes) => {
                    transcript.append_message(*label, &bytes);
                }
                VRFValue::U64(val) => {
                    transcript.append_u64(*label, val);
                }
            }
        }

        transcript
    }

    pub struct StaticGuard<T: ?Sized + 'static>(&'static T);

    impl StaticGuard<[u8]> {
        pub fn from_slice(slice: &[u8]) -> Self {
            slice.to_vec().into_boxed_slice().into()
        }
    }

    impl<T: ?Sized> Drop for StaticGuard<T> {
        fn drop(&mut self) {
            unsafe {
                crate::Box::from_raw(self.0 as *const T as *mut T);
            }
        }
    }

    impl<T: ?Sized> core::ops::Deref for StaticGuard<T> {
        type Target = &'static T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: ?Sized> From<crate::Box<T>> for StaticGuard<T> {
        fn from(b: crate::Box<T>) -> Self {
            let leak = crate::Box::leak(b);
            Self(leak)
        }
    }

    #[derive(Debug, Clone)]
    pub enum VRFError {
        Length(usize),
        UnknownVariant(u8),
    }

    impl From<ArrayError<usize>> for VRFError {
        fn from(err: ArrayError<usize>) -> Self {
            match err {
                ArrayError::Length(len) | ArrayError::Serde(len) => Self::Length(len),
            }
        }
    }

    impl From<usize> for VRFError {
        fn from(err: usize) -> Self {
            Self::Length(err)
        }
    }

    impl<'de> Deserialize<'de> for VRFValue<'de> {
        type Error = VRFError;

        fn deserialize(input: &'de [u8]) -> Result<Self, Self::Error> {
            let variant = input.get(0).cloned().ok_or(VRFError::Length(1))?;

            match variant {
                0 => {
                    let bytes: &[u8] =
                        Deserialize::deserialize(&input[1..]).map_err(|e| VRFError::Length(e))?;
                    Ok(Self::Bytes(bytes))
                }
                1 => {
                    let bytes: [u8; 8] = DeserializeOwned::deserialize_owned(&input[0..])?;
                    let num = u64::from_le_bytes(bytes);
                    Ok(Self::U64(num))
                }
                err => Err(VRFError::UnknownVariant(err)),
            }
        }
    }

    impl From<Tuple2Error<usize, VRFError>> for VRFError {
        fn from(err: Tuple2Error<usize, VRFError>) -> Self {
            match err {
                Tuple2Error::Length(l) | Tuple2Error::ErrorA(l) => VRFError::Length(l),
                Tuple2Error::ErrorB(e) => e,
            }
        }
    }

    impl<'de> Deserialize<'de> for VRFData<'de> {
        type Error = VRFError;

        fn deserialize(mut input: &'de [u8]) -> Result<Self, Self::Error> {
            //get the label
            let label: &[u8] = Deserialize::deserialize(input)?;
            crate::util::advance_slice(&mut input, label.len()).unwrap();

            //get the number of items
            let n_items: [u8; 8] = DeserializeOwned::deserialize_owned(input)?;
            let n_items = u64::from_le_bytes(n_items) as usize;
            crate::util::advance_slice(&mut input, 8).unwrap();

            //prepare an items vector
            let mut items = crate::Vec::with_capacity(n_items);
            for _ in 0..n_items {
                //get item length (in bytes)
                let item_size: [u8; 8] = DeserializeOwned::deserialize_owned(input)?;
                let item_size = u64::from_le_bytes(item_size) as usize;

                //deserialize item
                let item = Deserialize::deserialize(input)?;
                crate::util::advance_slice(&mut input, item_size).unwrap(); //and advance input by len

                //add to collection
                items.push(item);
            }

            Ok(Self { label, items })
        }
    }
}
pub use vrf::VRFData;
