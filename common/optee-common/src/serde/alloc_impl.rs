use crate::HasKeysPair;

use super::*;

use std::convert::Infallible;

impl<T: Serialize> Serialize for [T] {
    type Error = T::Error;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let len = self.len() as u64;

        let mut vec = vec![0; 8];
        vec[..8].copy_from_slice(&len.to_le_bytes()[..]);
        for item in self.iter() {
            vec.append(&mut item.serialize()?);
        }

        Ok(vec)
    }
}

impl Serialize for &str {
    type Error = ();

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        self.as_bytes().serialize()
    }
}

pub(crate) mod vec {
    use super::*;

    impl<T: Serialize> Serialize for Vec<T> {
        type Error = T::Error;

        fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
            let len = self.len() as u64;

            //number of vectors
            let mut vec = vec![0; 8];
            vec[..8].copy_from_slice(&len.to_le_bytes()[..]);

            //all the inner items in sequence
            for item in self.iter() {
                vec.append(&mut item.serialize()?);
            }

            Ok(vec)
        }
    }

    impl<T: Serialize> Serialize for &Vec<T> {
        type Error = T::Error;

        fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
            self.serialize()
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
                array.copy_from_slice(input);
                u64::from_le_bytes(array) as usize
            };

            let mut total_size = 8;
            let mut container = Vec::with_capacity(n_items);

            for _ in 0..n_items {
                let (size, item) = T::deserialize_variable(&input[total_size..])?;

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
        pair.append(&mut self.public_key.as_slice().serialize().unwrap());

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

        let total_bytes = 4 + 8 + key_len;
        //check for key_type + len + key_len
        if input.len() < total_bytes {
            return Err(());
        }

        let key: &[u8] = Deserialize::deserialize(&input[12..]).unwrap();
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

mod cow {
    use super::*;
    use std::borrow::{Cow, ToOwned};

    impl<'c, T> Serialize for Cow<'c, [T]>
    where
        [T]: ToOwned + Serialize<Error = <<[T] as ToOwned>::Owned as Serialize>::Error>,
        <[T] as ToOwned>::Owned: Serialize,
    {
        type Error = <[T] as Serialize>::Error;

        fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
            match self {
                Cow::Borrowed(borr) => borr.serialize(),
                Cow::Owned(own) => own.serialize(),
            }
        }
    }
}

pub(crate) mod tuple2 {
    use super::*;

    #[derive(Debug, Clone)]
    pub enum Tuple2Error<AE, BE> {
        ErrorA(AE),
        ErrorB(BE),
    }

    impl<A: Serialize, B: Serialize> Serialize for (A, B) {
        type Error = Tuple2Error<A::Error, B::Error>;

        fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
            let mut a = self.0.serialize().map_err(Tuple2Error::ErrorA)?;
            let mut b = self.1.serialize().map_err(Tuple2Error::ErrorB)?;

            let mut v = Vec::with_capacity(a.len() + b.len());
            v.append(&mut a);
            v.append(&mut b);

            Ok(v)
        }
    }

    impl<A: DeserializeVariable, B: DeserializeVariable> DeserializeVariable for (A, B) {
        type ErrorVariable = Tuple2Error<A::ErrorVariable, B::ErrorVariable>;

        fn deserialize_variable(input: &[u8]) -> Result<(usize, Self), Self::ErrorVariable> {
            let (size_a, a) =
                DeserializeVariable::deserialize_variable(input).map_err(Tuple2Error::ErrorA)?;
            let (size_b, b) = DeserializeVariable::deserialize_variable(&input[size_a..])
                .map_err(Tuple2Error::ErrorB)?;

            Ok(((size_a + size_b), (a, b)))
        }
    }
}
