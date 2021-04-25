//!Series of traits that define what kind of serialization / deserialization is possible (based on context)

use std::prelude::v1::*;

///Indicates that when serialized the type has a fixed size
pub trait SerializeFixed {
    type ErrorFixed;

    fn len() -> usize;

    ///Will serialize self into `dest`, the function is allowed to panic if dest.len() < len
    fn serialize_fixed(&self, dest: &mut [u8]) -> Result<(), Self::ErrorFixed>;
}

impl<T: SerializeFixed> SerializeFixed for &T {
    type ErrorFixed = T::ErrorFixed;

    fn len() -> usize {
        T::len()
    }

    fn serialize_fixed(&self, dest: &mut [u8]) -> Result<(), Self::ErrorFixed> {
        T::serialize_fixed(self, dest)
    }
}

///Indicates that when deserialized the type copies memory in the stack
pub trait DeserializeOwned: SerializeFixed + Sized {
    type ErrorOwned;

    fn deserialize_owned(input: &[u8]) -> Result<Self, Self::ErrorOwned>;
}

///Indicates that when deserialized the type borrows memory
pub trait Deserialize<'de>: Sized {
    type Error;

    fn deserialize(input: &'de [u8]) -> Result<Self, Self::Error>;
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for T {
    type Error = <Self as DeserializeOwned>::ErrorOwned;

    fn deserialize(input: &'de [u8]) -> Result<Self, Self::Error> {
        Self::deserialize_owned(input)
    }
}

#[cfg(feature = "alloc")]
///This type needs a variable amout of storage when deserializing
pub trait DeserializeVariable: Sized {
    type ErrorVariable;

    fn deserialize_variable(input: &[u8]) -> Result<(usize, Self), Self::ErrorVariable>;
}

#[cfg(feature = "alloc")]
impl<T: DeserializeOwned> DeserializeVariable for T {
    type ErrorVariable = T::ErrorOwned;

    fn deserialize_variable(input: &[u8]) -> Result<(usize, Self), Self::ErrorVariable> {
        T::deserialize_owned(input).map(|t| (T::len(), t))
    }
}

#[cfg(feature = "alloc")]
///This type needs a variable amount of storage when serializing
pub trait Serialize {
    type Error;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error>;
}

#[cfg(feature = "alloc")]
impl<T: SerializeFixed> Serialize for T {
    type Error = <Self as SerializeFixed>::ErrorFixed;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let len = <Self as SerializeFixed>::len();

        let mut v = vec![0; len];
        self.serialize_fixed(&mut v)?;

        Ok(v)
    }
}

mod common_impl;

mod schnorrkel_impl;

#[cfg(feature = "alloc")]
mod alloc_impl;

#[cfg(feature = "alloc")]
pub use alloc_impl::vec::VecError;

#[cfg(feature = "sp")]
mod sp;

mod core_impl;
pub use core_impl::{ArrayError, StrError};

#[derive(Debug, Clone)]
pub enum Tuple2Error<AE, BE> {
    Length(usize),
    ErrorA(AE),
    ErrorB(BE),
}
