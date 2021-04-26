use super::*;

impl<'de> Deserialize<'de> for &'de [u8] {
    type Error = usize;

    fn deserialize(mut input: &'de [u8]) -> Result<Self, Self::Error> {
        if input.len() < 8 {
            return Err(8);
        }

        let len = {
            let mut array = [0; 8];
            array.copy_from_slice(&input[..8]);
            input = &input[8..];
            u64::from_le_bytes(array) as usize
        };

        if input.len() < len {
            return Err(len);
        }

        Ok(&input[..len as usize])
    }
}

#[derive(Debug, Clone)]
pub enum StrError {
    /// provided slice wasn't a valid utf8 string
    Utf8(std::str::Utf8Error),
    /// there was an error with the length of the provided slice
    Length(usize),
}

impl<'de> Deserialize<'de> for &'de str {
    type Error = StrError;

    fn deserialize(input: &'de [u8]) -> Result<Self, Self::Error> {
        let input: &[u8] = Deserialize::deserialize(input).map_err(StrError::Length)?;

        std::str::from_utf8(input).map_err(StrError::Utf8)
    }
}

impl SerializeFixed for u8 {
    type ErrorFixed = usize;

    fn len() -> usize {
        1
    }

    fn serialize_fixed(&self, dest: &mut [u8]) -> Result<(), Self::ErrorFixed> {
        *dest.get_mut(0).ok_or(1usize)? = *self;
        Ok(())
    }
}

impl DeserializeOwned for u8 {
    type ErrorOwned = usize;

    fn deserialize_owned(input: &[u8]) -> Result<Self, Self::ErrorOwned> {
        input.get(0).cloned().ok_or(1)
    }
}
///Error enum when dealing with arrays
#[derive(Debug, Clone)]
pub enum ArrayError<E> {
    ///Lenght of source or destination does not match the array length
    Length(usize),
    ///Error serializing or deserializing
    Serde(E),
}

impl<E> From<E> for ArrayError<E> {
    fn from(from: E) -> Self {
        Self::Serde(from)
    }
}

impl<T: SerializeFixed, const N: usize> SerializeFixed for [T; N] {
    type ErrorFixed = ArrayError<T::ErrorFixed>;

    fn len() -> usize {
        T::len() * N
    }

    fn serialize_fixed(&self, dest: &mut [u8]) -> Result<(), Self::ErrorFixed> {
        if dest.len() < T::len() * N {
            return Err(ArrayError::Length(T::len() * N));
        }

        let chunks = dest.chunks_exact_mut(T::len());

        for (item, slice) in self.iter().zip(chunks) {
            item.serialize_fixed(slice)?;
        }

        Ok(())
    }
}

impl<T: SerializeFixed, const N: usize> Serialize for [T; N] {
    type Error = T::ErrorFixed;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let len = T::len() * N;
        let mut v = vec![0; len];

        match <[T; N] as SerializeFixed>::serialize_fixed(self, &mut v) {
            Ok(_) => Ok(v),
            Err(ArrayError::Length(_)) => unreachable!(),
            Err(ArrayError::Serde(s)) => Err(s),
        }
    }
}

impl<T: DeserializeOwned, const N: usize> DeserializeOwned for [T; N] {
    type ErrorOwned = ArrayError<T::ErrorOwned>;

    fn deserialize_owned(input: &[u8]) -> Result<Self, Self::ErrorOwned> {
        use std::mem::{transmute_copy, MaybeUninit};

        if input.len() < T::len() * N {
            return Err(ArrayError::Length(T::len() * N));
        }

        let mut container: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        for (i, input) in input.chunks_exact(T::len()).enumerate().take(N) {
            let item = T::deserialize(input)?;

            //we override the uninitialized data with initialized data
            container[i] = MaybeUninit::new(item);
        }

        //SAFETY: all items have been initialized
        Ok(unsafe { transmute_copy(&container) })
    }
}

mod tuple2 {
    use super::*;

    impl<'de, A: Deserialize<'de>, B: Deserialize<'de>> Deserialize<'de> for (A, B) {
        type Error = Tuple2Error<A::Error, B::Error>;

        fn deserialize(input: &'de [u8]) -> Result<Self, Self::Error> {
            let midpoint: [u8; 8] = DeserializeOwned::deserialize_owned(input)?;
            let midpoint = u64::from_le_bytes(midpoint) as usize;
            let input = &input[8..];

            let a = Deserialize::deserialize(&input[..midpoint]).map_err(Tuple2Error::ErrorA)?;
            let b =
                Deserialize::deserialize(&input[midpoint..]).map_err(Tuple2Error::ErrorB)?;

            Ok((a, b))
        }
    }

    // impl<A: SerializeFixed, B: SerializeFixed> SerializeFixed for (A, B) {
    //     type ErrorFixed = Tuple2Error<A::ErrorFixed, B::ErrorFixed>;

    //     fn len() -> usize {
    //         8 + A::len() + B::len()
    //     }

    //     fn serialize_fixed(&self, dest: &mut [u8]) -> Result<(), Self::ErrorFixed> {
    //         if dest.len() < Self::len() {
    //             return Err(Tuple2Error::Length(Self::len()));
    //         }

    //         let midpoint = A::len() as u64;
    //         midpoint.to_le_bytes().serialize_fixed(dest).unwrap();

    //         self.0.serialize_fixed(dest).map_err(Tuple2Error::ErrorA)?;
    //         self.1.serialize_fixed(dest).map_err(Tuple2Error::ErrorB)?;

    //         Ok(())
    //     }
    // }
}
