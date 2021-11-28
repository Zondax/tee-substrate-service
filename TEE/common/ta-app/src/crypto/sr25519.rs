use schnorrkel::{
    keys,
    vrf::{VRFPreOut, VRFProof},
    Signature, SignatureError,
};

use crate::util::CSPRNG;

#[derive(Clone)]
pub struct Keypair(keys::Keypair);

impl Keypair {
    pub fn generate<C: CSPRNG>(rng: &mut C) -> Self {
        Self(keys::Keypair::generate_with(rng))
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes.len() {
            32 => Some(Self(
                keys::MiniSecretKey::from_bytes(bytes)
                    .ok()?
                    .expand_to_keypair(keys::ExpansionMode::Ed25519),
            )),
            64 => Some(Self(keys::SecretKey::from_bytes(bytes).ok()?.to_keypair())),
            _ => None,
        }
    }

    pub fn public(&self) -> &[u8] {
        self.0.public.as_ref()
    }

    fn get_transcript(msg: &[u8]) -> merlin::Transcript {
        let mut t = merlin::Transcript::new(b"SigningContext");
        t.append_message(b"", b"substrate"); //ctx
        t.append_message(b"sign-bytes", msg);
        t
    }

    pub fn sign<C: CSPRNG>(&self, rng: &mut C, msg: &[u8]) -> [u8; 64] {
        let t = Self::get_transcript(msg);
        let t = schnorrkel::context::attach_rng(t, rng);

        self.0.sign(t).to_bytes()
    }

    pub fn vrf_sign<C: CSPRNG>(&self, rng: &mut C, data: vrf::VRFData<'_>) -> crate::Vec<u8> {
        let t = vrf::make_transcript(data);

        //see: https://github.com/w3f/schnorrkel/issues/70
        let extra = {
            let t = merlin::Transcript::new(b"VRF");
            schnorrkel::context::attach_rng(t, rng)
        };

        let (inout, proof, _) = self.0.vrf_sign_extra(t, extra);
        let preout = inout.to_preout();

        let preout = preout.to_bytes();
        let proof = proof.to_bytes();

        let midpoint = preout.len() as u64;
        let midpoint = midpoint.to_le_bytes();

        //VRFSignature serialized
        [&midpoint[..], &preout[..], &proof[..]].concat()
    }
}

#[derive(Debug)]
pub struct PublicKey(keys::PublicKey);

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SignatureError> {
        keys::PublicKey::from_bytes(bytes).map(Self)
    }

    pub fn verify(&self, msg: &[u8], sig: &[u8; 64]) -> bool {
        let sig = match Signature::from_bytes(&sig[..]) {
            Ok(sig) => sig,
            Err(_) => return false,
        };

        let transcript = Keypair::get_transcript(msg);

        self.0.verify(transcript, &sig).is_ok()
    }

    pub fn verify_vrf(&self, data: vrf::VRFData<'_>, sig: &[u8]) -> bool {
        if sig.len() < 8 + 32 + 64 {
            return false;
        }

        let midpoint: [u8; 8] = match optee_common::DeserializeOwned::deserialize_owned(&sig[..8]) {
            Err(_) => return false,
            Ok(midpoint) => midpoint,
        };
        let midpoint = u64::from_le_bytes(midpoint) as usize;
        let sig = &sig[8..];

        let preout = &sig[..midpoint];
        let preout = match VRFPreOut::from_bytes(preout) {
            Err(_) => return false,
            Ok(preout) => preout,
        };

        let proof = &sig[midpoint..];
        let proof = match VRFProof::from_bytes(proof) {
            Err(_) => return false,
            Ok(proof) => proof,
        };

        let t = vrf::make_transcript(data);

        self.0.vrf_verify(t, &preout, &proof).is_ok()
    }
}

impl From<&Keypair> for PublicKey {
    fn from(pair: &Keypair) -> Self {
        Self(pair.0.public)
    }
}

mod vrf {
    use merlin::Transcript;
    use optee_common::{
        serde::{ArrayError, Tuple2Error},
        Deserialize, DeserializeOwned,
    };

    #[derive(Clone, Debug)]
    pub enum VRFValue<'de> {
        Bytes(&'de [u8]),
        U64(u64),
    }

    #[derive(Clone, Debug)]
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
                    let bytes: [u8; 8] = DeserializeOwned::deserialize_owned(&input[1..])?;
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
            crate::util::advance_slice(&mut input, 8 + label.len()).unwrap();

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
                let item = Deserialize::deserialize(&input[8..8 + item_size])?;
                crate::util::advance_slice(&mut input, 8 + item_size).unwrap(); //and advance input by len

                //add to collection
                items.push(item);
            }

            Ok(Self { label, items })
        }
    }
}
pub use vrf::VRFData;
