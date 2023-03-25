// A quasi-bump-allocated map that stores anything that can be represented by Any on the heap
// loosely based on
// http://benfalk.com/blog/2022/02/27/rust-hashmap-to-store-anything/
// and https://twitter.com/nercury/status/883046507225776134
// and async-graphql's Context
use std::{
  any::{Any, TypeId},
  collections::HashMap,
  hash::Hash,
  sync::Arc,
};

type HashKey<K> = (K, TypeId);
type Anything = Arc<dyn Any + Send + Sync>;

#[derive(Debug, Default)]
pub struct AnyStore<K: Eq + Hash> {
  storage: Vec<Anything>,
  mapping: HashMap<HashKey<K>, usize>,
}

#[allow(dead_code)]
impl<K: Eq + Hash + Clone> AnyStore<K> {
  /// Creates a new hashmap that can store
  /// any data which can be tagged with the
  /// `Any` trait.
  pub fn new() -> Self {
    Self {
      storage: vec![],
      mapping: HashMap::new(),
    }
  }

  /// Creates a new hashmap that can store
  /// at least the capacity given.
  pub fn new_with_capacity(capacity: usize) -> Self {
    Self {
      storage: Vec::with_capacity(capacity),
      mapping: HashMap::with_capacity(capacity),
    }
  }

  pub fn get<V: Any + Send + Sync>(&self, key: K) -> Option<Arc<V>> {
    let index = self.mapping.get(&(key, TypeId::of::<V>())).copied();
    index.and_then(|index| self.get_downcast(index))
  }

  pub fn get_or_insert<V: Any + Send + Sync>(&mut self, key: K, val: V) -> Arc<V> {
    let index = self.mapping.get(&(key.clone(), TypeId::of::<V>())).copied();

    match index {
      Some(index) => self.get_downcast(index).unwrap(),
      None => {
        let index = self.storage.len();
        self.storage.push(Arc::new(val));
        self.mapping.insert((key, TypeId::of::<V>()), index);
        self.get_downcast(index).unwrap()
      }
    }
  }

  fn get_downcast<V: Any + Send + Sync>(&self, index: usize) -> Option<Arc<V>> {
    self
      .storage
      .get(index)
      .and_then(|anything| anything.clone().downcast::<V>().ok())
  }
}
