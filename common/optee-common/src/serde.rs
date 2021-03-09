//!Series of traits that define what kind of serialization / deserialization is possible (based on context)

use std::prelude::v1::*;

///Indicates that when serialized the type has a fixed size
pub trait SerializeFixed {
    type ErrorFixed;

    fn len() -> usize;

    ///Will serialize self into `dest`, the function is allowed to panic if dest.len() < len
    fn serialize_fixed(&self, dest: &mut [u8]) -> Result<(), Self::ErrorFixed>;
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

mod schnorrkel_impl;

#[cfg(feature = "alloc")]
mod alloc_impl;

mod core_impl;
