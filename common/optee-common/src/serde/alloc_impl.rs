use crate::HasKeysPair;

use super::*;

use std::convert::Infallible;

impl<T: Serialize> Serialize for [T] {
    type Error = T::Error;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let len = self.len() as u64;

        //number of items
        let mut vec = len.to_le_bytes().to_vec();

        for item in self.iter() {
            let mut item = item.serialize()?;
            let size = item.len() as u64;

            vec.extend_from_slice(&size.to_le_bytes()[..]);
            vec.append(&mut item);
        }

        Ok(vec)
    }
}

impl Serialize for &[u8] {
    type Error = Infallible;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let len = self.len() as u64;

        let mut vec = vec![0; 8];
        //number of items
        vec[..8].copy_from_slice(&len.to_le_bytes()[..]);

        vec.extend_from_slice(self);

        Ok(vec)
    }
}

impl Serialize for &str {
    type Error = usize;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let slice = self.as_bytes();
        let len = slice.len();
        slice.serialize().map_err(|_| len)
    }
}

impl Serialize for u8 {
    type Error = Infallible;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        Ok(vec![*self])
    }
}

pub(crate) mod vec {
    use super::*;

    // impl<T: Serialize> Serialize for Vec<T> {
    //     type Error = T::Error;

    //     fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
    //         <[T] as Serialize>::serialize(&self.as_slice())
    //     }
    // }

    impl Serialize for Vec<u8> {
        type Error = Infallible;

        fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
            let len = self.len() as u64;

            Ok([&len.to_le_bytes()[..], self.as_slice()].concat())
        }
    }

    impl Serialize for Vec<Vec<u8>> {
        type Error = Infallible;

        fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
            let mut v = (self.len() as u64).to_le_bytes().to_vec();
            for inner in self.iter() {
                v.append(&mut inner.serialize().unwrap())
            }

            Ok(v)
        }
    }

    impl Serialize for Vec<HasKeysPair> {
        type Error = Infallible;

        fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
            let mut v = (self.len() as u64).to_le_bytes().to_vec();
            for inner in self.iter() {
                v.append(&mut inner.serialize()?)
            }

            Ok(v)
        }
    }

    impl<T> Serialize for &Vec<T>
    where
        Vec<T>: Serialize,
    {
        type Error = <Vec<T> as Serialize>::Error;

        fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
            <Vec<T> as Serialize>::serialize(self)
        }
    }

    ///Error enum when dealing with vecs
    #[derive(Debug, Clone)]
    pub enum VecError<E> {
        ///Lenght of source does not match the advertised length
        Length(usize),
        ///Error serializing or deserializing
        Serde(E),
    }

    impl<E> From<E> for VecError<E> {
        fn from(from: E) -> Self {
            Self::Serde(from)
        }
    }

    impl<T: DeserializeVariable> DeserializeVariable for Vec<T> {
        type ErrorVariable = VecError<T::ErrorVariable>;

        fn deserialize_variable(input: &[u8]) -> Result<(usize, Self), Self::ErrorVariable> {
            if input.len() < 8 {
                return Err(VecError::Length(8));
            }

            let n_items = {
                let mut array = [0; 8];
                array.copy_from_slice(&input[..8]);
                u64::from_le_bytes(array) as usize
            };

            let mut total_size = 8;
            let mut container = Vec::new();

            for _ in 0..n_items {
                let input = input
                    .get(total_size..)
                    .ok_or(Self::ErrorVariable::Length(total_size))?;

                let (size, item) = T::deserialize_variable(input)?;

                container.push(item);
                total_size += size;
            }

            Ok((total_size, container))
        }
    }
}

impl Serialize for HasKeysPair {
    type Error = Infallible;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        //create vec for key_type : len : key
        let mut pair = vec![0; 4];
        self.key_type.serialize_fixed(&mut pair[..4]).unwrap();
        pair.append(&mut (&self.public_key.as_slice()).serialize().unwrap());

        Ok(pair)
    }
}

impl DeserializeVariable for HasKeysPair {
    type ErrorVariable = ();

    fn deserialize_variable(input: &[u8]) -> Result<(usize, Self), Self::ErrorVariable> {
        //check for key_type + len
        if input.len() < 4 + 8 {
            return Err(());
        }
        let key_type: [u8; 4] = DeserializeOwned::deserialize_owned(&input[..4]).unwrap();

        let key_len = {
            let mut array = [0; 8];
            array.copy_from_slice(&input[4..12]);
            u64::from_le_bytes(array) as usize
        };

        let total_bytes = key_len.checked_add(4 + 8).ok_or(())?;
        //check for key_type + len + key_len
        if input.len() < total_bytes {
            return Err(());
        }

        let key: &[u8] = Deserialize::deserialize(&input[4..]).unwrap();
        let public_key = key.to_vec();

        Ok((
            total_bytes,
            HasKeysPair {
                key_type,
                public_key,
            },
        ))
    }
}

impl Serialize for crate::CryptoAlgo {
    type Error = Infallible;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let len = Self::len();

        let mut v = vec![0; len];
        self.serialize_fixed(&mut v).unwrap();

        Ok(v)
    }
}

mod cow {
    use super::*;
    use std::borrow::{Cow, ToOwned};

    // impl<'c, T> Serialize for Cow<'c, [T]>
    // where
    //     for<'a> &'a [T]: Serialize<Error = <<[T] as ToOwned>::Owned as Serialize>::Error>,
    //     [T]: ToOwned,
    //     <[T] as ToOwned>::Owned: Serialize,
    // {
    //     type Error = <<[T] as ToOwned>::Owned as Serialize>::Error;

    //     fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
    //         match self {
    //             Cow::Borrowed(borr) => borr.serialize(),
    //             Cow::Owned(own) => own.serialize(),
    //         }
    //     }
    // }

    // impl<'c, T> Serialize for Cow<'c, [T]>
    // where
    //     [T]: ToOwned + Serialize<Error = <<[T] as ToOwned>::Owned as Serialize>::Error>,
    //     <[T] as ToOwned>::Owned: Serialize,
    // {
    //     type Error = <<[T] as ToOwned>::Owned as Serialize>::Error;

    //     fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
    //         match self {
    //             Cow::Borrowed(borr) => borr.serialize(),
    //             Cow::Owned(own) => own.serialize(),
    //         }
    //     }
    // }

    impl<'c> Serialize for Cow<'c, [u8]> {
        type Error = Infallible;

        fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
            match self {
                Cow::Borrowed(borr) => (&borr).serialize(),
                Cow::Owned(own) => (&own).serialize(),
            }
        }
    }
}

mod tuple2 {
    use super::*;

    // impl<A: Serialize, B: Serialize> Serialize for (A, B) {
    //     type Error = Tuple2Error<A::Error, B::Error>;

    //     fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
    //         let mut a = self.0.serialize().map_err(Tuple2Error::ErrorA)?;
    //         let mut b = self.1.serialize().map_err(Tuple2Error::ErrorB)?;

    //         let mut v = Vec::with_capacity(a.len() + b.len());
    //         let midpoint = a.len() as u64;
    //         v.extend_from_slice(&midpoint.to_le_bytes()[..]);
    //         v.append(&mut a);
    //         v.append(&mut b);

    //         Ok(v)
    //     }
    // }

    impl<A, B> From<ArrayError<usize>> for Tuple2Error<A, B> {
        fn from(err: ArrayError<usize>) -> Self {
            match err {
                ArrayError::Length(len) | ArrayError::Serde(len) => Self::Length(len),
            }
        }
    }

    // impl<A: DeserializeVariable, B: DeserializeVariable> DeserializeVariable for (A, B) {
    //     type ErrorVariable = Tuple2Error<A::ErrorVariable, B::ErrorVariable>;

    //     fn deserialize_variable(input: &[u8]) -> Result<(usize, Self), Self::ErrorVariable> {
    //         let midpoint: [u8; 8] = DeserializeOwned::deserialize_owned(input)?;
    //         let midpoint = u64::from_le_bytes(midpoint) as usize;

    //         let (size_a, a) = DeserializeVariable::deserialize_variable(&input[8..])
    //             .map_err(Tuple2Error::ErrorA)?;

    //         if size_a != midpoint {
    //             return Err(Tuple2Error::Length(midpoint));
    //         }

    //         let (size_b, b) = DeserializeVariable::deserialize_variable(&input[8 + size_a..])
    //             .map_err(Tuple2Error::ErrorB)?;

    //         Ok(((size_a + size_b), (a, b)))
    //     }
    // }
}
