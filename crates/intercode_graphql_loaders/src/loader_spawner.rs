use std::{collections::HashMap, hash::Hash, marker::PhantomData, sync::Arc};

use async_graphql::{
  dataloader::{DataLoader, Loader},
  futures_util::lock::Mutex,
};
use core::time::Duration;
use seawater::ConnectionWrapper;

pub struct LoaderSpawner<
  Key: Hash + Clone + Eq,
  LoaderKey: Send + Sync + Hash + Eq + Clone + 'static,
  L: Loader<LoaderKey>,
> {
  loaders_by_key: Arc<Mutex<HashMap<Key, Arc<DataLoader<L>>>>>,
  db: ConnectionWrapper,
  delay: Duration,
  build_loader: Box<dyn Fn(ConnectionWrapper, Key) -> L + Send + Sync>,
  _phantom: PhantomData<LoaderKey>,
}

impl<
    Key: Hash + Clone + Eq,
    LoaderKey: Send + Sync + Hash + Eq + Clone + 'static,
    L: Loader<LoaderKey>,
  > LoaderSpawner<Key, LoaderKey, L>
{
  pub fn new<F>(db: ConnectionWrapper, delay: Duration, build_loader: F) -> Self
  where
    F: Fn(ConnectionWrapper, Key) -> L + Send + Sync + 'static,
  {
    LoaderSpawner {
      loaders_by_key: Default::default(),
      db,
      delay,
      build_loader: Box::new(build_loader),
      _phantom: Default::default(),
    }
  }

  pub async fn get(&self, key: Key) -> Arc<DataLoader<L>> {
    let mut lock = self.loaders_by_key.lock().await;
    let loader = lock.entry(key.clone()).or_insert_with(|| {
      Arc::new(
        DataLoader::new((self.build_loader)(self.db.clone(), key), tokio::spawn).delay(self.delay),
      )
    });

    loader.clone()
  }
}
