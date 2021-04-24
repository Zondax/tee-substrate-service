use core::borrow::Borrow;
use std::convert::Infallible;

use super::*;
use super::{
    alloc_impl::{tuple2::Tuple2Error, vec::VecError},
    core_impl::ArrayError,
};

use sp_keystore::vrf::{VRFTranscriptData, VRFTranscriptValue};

impl Serialize for VRFTranscriptValue {
    type Error = Infallible;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        match self {
            VRFTranscriptValue::Bytes(bytes) => Ok([&[0], bytes.as_slice()].concat()),
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
                    DeserializeVariable::deserialize_variable(&input[0..])?;
                Ok((len, Self::Bytes(bytes)))
            }
            1 => {
                let (len, num): (_, [u8; 8]) =
                    DeserializeVariable::deserialize_variable(&input[0..])?;
                let num = u64::from_le_bytes(num);
                Ok((len, Self::U64(num)))
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
        v.append(&mut label.serialize().unwrap());

        //all errors are Infallible
        v.append(&mut self.items.serialize().unwrap());

        Ok(v)
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
                | Tuple2Error::ErrorB(VRFTranscriptValueError::Length(len)),
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
