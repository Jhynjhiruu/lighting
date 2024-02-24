use std::any::Any;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;

mod passthrough;

use self::passthrough::PassthroughHasherBuilder;

type _AssetKey = u64;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AssetKey<T> {
    key: _AssetKey,
    _marker: PhantomData<T>,
}

impl<T> Clone for AssetKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for AssetKey<T> {}

struct AssetServer {
    map: HashMap<_AssetKey, Box<dyn Any>, PassthroughHasherBuilder>,
    builder: PassthroughHasherBuilder,
}

unsafe impl Send for AssetServer {}
unsafe impl Sync for AssetServer {}

static mut SERVER: AssetServer = AssetServer {
    map: HashMap::with_hasher(PassthroughHasherBuilder),
    builder: PassthroughHasherBuilder,
};

pub fn add_asset<T: Hash, U: 'static>(key: T, value: U) -> AssetKey<U> {
    let key = unsafe { SERVER.builder.hash_one(&key) };

    unsafe { SERVER.map.insert(key, Box::new(value)) };

    AssetKey {
        key,
        _marker: PhantomData,
    }
}

pub fn retrieve_asset<T: 'static>(key: &AssetKey<T>) -> &T {
    unsafe { SERVER.map[&key.key].downcast_ref_unchecked() }
}
