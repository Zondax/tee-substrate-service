use optee_common::{DeserializeOwned, Serialize};

///Trait that marks a type as a storage object
pub trait Object: Serialize + DeserializeOwned {
    /// The object ID
    type ID: ObjID;
}

///Marker trait for types usable as ObjID
pub trait ObjID: Serialize + DeserializeOwned {}

pub trait Storage: Sized {
    type Iter: StorageEnumerator<Store = Self>;

    fn store<T: Object>(&mut self, val: &T) -> T::ID;

    fn retrieve<T: Object>(&mut self, id: T::ID) -> Option<T>;

    fn delete<T: Object>(&mut self, id: T::ID) -> bool;

    fn rename<T: Object>(&mut self, old_id: T::ID, new_id: T::ID) -> bool;

    fn iter(&self) -> Self::Iter;
}

pub trait StorageEnumerator {
    type Store: Storage;

    fn next<T: Object>(&mut self) -> Option<(T::ID, T)>;
}
