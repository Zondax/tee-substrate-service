use core::borrow::Borrow;
use std::convert::Infallible;

use super::*;
use super::{alloc_impl::vec::VecError, core_impl::ArrayError};

use sp_keystore::vrf::{
    SignatureError, VRFOutput, VRFProof, VRFSignature, VRFTranscriptData, VRFTranscriptValue,
};

impl Serialize for VRFTranscriptValue {
    type Error = Infallible;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        match self {
            VRFTranscriptValue::Bytes(bytes) => {
                let len = bytes.len() as u64;
                let len = len.to_le_bytes();

                Ok([&[0], &len[..], bytes.as_slice()].concat())
            }
            VRFTranscriptValue::U64(num) => Ok([&[1], &num.to_le_bytes()[..]].concat()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum VRFTranscriptValueError {
    Length(usize),
    UnknownVariant(u8),
}

impl From<VecError<usize>> for VRFTranscriptValueError {
    fn from(err: VecError<usize>) -> Self {
        match err {
            VecError::Length(len) | VecError::Serde(len) => Self::Length(len),
        }
    }
}

impl From<ArrayError<usize>> for VRFTranscriptValueError {
    fn from(err: ArrayError<usize>) -> Self {
        match err {
            ArrayError::Length(len) | ArrayError::Serde(len) => Self::Length(len),
        }
    }
}

impl DeserializeVariable for VRFTranscriptValue {
    type ErrorVariable = VRFTranscriptValueError;

    fn deserialize_variable(input: &[u8]) -> Result<(usize, Self), Self::ErrorVariable> {
        let variant = input
            .get(0)
            .cloned()
            .ok_or(VRFTranscriptValueError::Length(1))?;

        match variant {
            0 => {
                let (len, bytes): (_, Vec<u8>) =
                    DeserializeVariable::deserialize_variable(&input[1..])?;
                Ok((1 + len, Self::Bytes(bytes)))
            }
            1 => {
                let num: [u8; 8] = DeserializeOwned::deserialize_owned(&input[1..])?;
                let num = u64::from_le_bytes(num);
                Ok((9, Self::U64(num)))
            }
            err => return Err(VRFTranscriptValueError::UnknownVariant(err)),
        }
    }
}

impl Serialize for VRFTranscriptData {
    type Error = Infallible;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let mut v = Vec::with_capacity(self.label.len());

        let label: &[u8] = self.label.borrow();
        v.append(&mut (&label).serialize().unwrap());

        //all errors are Infallible
        let mut items = self.items.serialize().unwrap();
        v.append(&mut items);

        Ok(v)
    }
}

impl<'c> Serialize for (std::borrow::Cow<'c, [u8]>, VRFTranscriptValue) {
    type Error = Infallible;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let mut a = self.0.serialize()?;
        let mut b = self.1.serialize()?;

        let midpoint = a.len() as u64;

        let mut v = midpoint.to_le_bytes().to_vec();
        v.append(&mut a);
        v.append(&mut b);

        Ok(v)
    }
}

impl<'c> DeserializeVariable for (Vec<u8>, VRFTranscriptValue) {
    type ErrorVariable = Tuple2Error<VecError<usize>, VRFTranscriptValueError>;

    fn deserialize_variable(input: &[u8]) -> Result<(usize, Self), Self::ErrorVariable> {
        let input = &input[8..]; //skip total size

        let midpoint: [u8; 8] = DeserializeOwned::deserialize_owned(input)?;
        let midpoint = u64::from_le_bytes(midpoint) as usize;

        let input = &input[8..];
        let (_, a) = DeserializeVariable::deserialize_variable(&input[..midpoint])
            .map_err(Tuple2Error::ErrorA)?;
        let (size_b, b) = DeserializeVariable::deserialize_variable(&input[midpoint..])
            .map_err(Tuple2Error::ErrorB)?;

        Ok(((8 + midpoint + size_b), (a, b)))
    }
}

#[derive(Debug, Clone)]
pub enum VRFTranscriptDataError {
    Length(usize),
    UnknownValueVariant(u8),
}

impl From<VecError<usize>> for VRFTranscriptDataError {
    fn from(err: VecError<usize>) -> Self {
        match err {
            VecError::Length(len) | VecError::Serde(len) => Self::Length(len),
        }
    }
}

impl From<VecError<Tuple2Error<VecError<usize>, VRFTranscriptValueError>>>
    for VRFTranscriptDataError
{
    fn from(err: VecError<Tuple2Error<VecError<usize>, VRFTranscriptValueError>>) -> Self {
        match err {
            VecError::Length(len)
            | VecError::Serde(
                Tuple2Error::ErrorA(VecError::Length(len) | VecError::Serde(len))
                | Tuple2Error::ErrorB(VRFTranscriptValueError::Length(len))
                | Tuple2Error::Length(len),
            ) => Self::Length(len),
            VecError::Serde(Tuple2Error::ErrorB(VRFTranscriptValueError::UnknownVariant(
                variant,
            ))) => Self::UnknownValueVariant(variant),
        }
    }
}

impl DeserializeVariable for VRFTranscriptData {
    type ErrorVariable = VRFTranscriptDataError;

    fn deserialize_variable(input: &[u8]) -> Result<(usize, Self), Self::ErrorVariable> {
        let (size_label, label): (_, Vec<u8>) = DeserializeVariable::deserialize_variable(input)?;
        let (size_items, items): (_, Vec<(Vec<u8>, VRFTranscriptValue)>) =
            DeserializeVariable::deserialize_variable(input)?;

        use std::borrow::Cow;

        let items = items
            .into_iter()
            .map(|(label, value)| (Cow::Owned(label), value))
            .collect();

        Ok((
            size_label + size_items,
            VRFTranscriptData {
                label: Cow::Owned(label),
                items,
            },
        ))
    }
}

impl SerializeFixed for VRFSignature {
    type ErrorFixed = usize;

    fn len() -> usize {
        8 + 32 + 64
    }

    fn serialize_fixed(&self, dest: &mut [u8]) -> Result<(), Self::ErrorFixed> {
        if dest.len() < Self::len() {
            return Err(Self::len());
        }

        let preout = self.output.to_bytes();
        let proof = self.proof.to_bytes();

        fn flatten_array_err(err: ArrayError<usize>) -> usize {
            match err {
                ArrayError::Length(l) | ArrayError::Serde(l) => l,
            }
        }

        let midpoint = preout.len() as u64;

        midpoint
            .to_le_bytes()
            .serialize_fixed(dest)
            .map_err(flatten_array_err)?;
        preout.serialize_fixed(dest).map_err(flatten_array_err)?;
        proof
            .serialize_fixed(&mut dest[preout.len()..])
            .map_err(flatten_array_err)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum VRFSignatureError {
    Length(usize),
    SignatureError(SignatureError),
}

impl From<ArrayError<usize>> for VRFSignatureError {
    fn from(err: ArrayError<usize>) -> Self {
        match err {
            ArrayError::Length(l) | ArrayError::Serde(l) => Self::Length(l),
        }
    }
}

impl From<SignatureError> for VRFSignatureError {
    fn from(err: SignatureError) -> Self {
        Self::SignatureError(err)
    }
}

impl DeserializeOwned for VRFSignature {
    type ErrorOwned = VRFSignatureError;

    fn deserialize_owned(input: &[u8]) -> Result<Self, Self::ErrorOwned> {
        if input.len() < Self::len() {
            return Err(VRFSignatureError::Length(Self::len()));
        }

        let midpoint: [u8; 8] = DeserializeOwned::deserialize_owned(input)?;
        let midpoint = u64::from_le_bytes(midpoint) as usize;
        if midpoint != 32 {
            return Err(VRFSignatureError::Length(midpoint));
        }

        let preout: [u8; 32] = DeserializeOwned::deserialize_owned(&input[8..midpoint])?;
        let proof: [u8; 64] = DeserializeOwned::deserialize_owned(&input[8 + midpoint..])?;

        let output = VRFOutput::from_bytes(&preout)?;
        let proof = VRFProof::from_bytes(&proof)?;

        Ok(Self { output, proof })
    }
}
