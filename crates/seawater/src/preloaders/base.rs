use std::{
  collections::HashMap,
  error::Error,
  fmt::{Debug, Display},
};

use crate::{DropResult, DropResultTrait, DropStoreID, LiquidDrop};
use async_trait::async_trait;
use liquid::ValueView;
use once_cell::race::OnceBox;
use tracing::{info_span, warn, warn_span};

use crate::{ConnectionWrapper, DropError, DropRef, DropStore};

use super::PreloaderResult;

#[derive(Debug, Clone)]
pub struct IncompleteBuilderError;

impl Display for IncompleteBuilderError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Builder parameters incomplete")
  }
}

impl Error for IncompleteBuilderError {}

pub trait LoaderResultToDropsFn<LoaderResult, FromDrop: LiquidDrop + 'static, ToDrop>
where
  Self: Fn(Option<LoaderResult>, DropRef<FromDrop>) -> Result<Vec<ToDrop>, DropError> + Send + Sync,
{
}

impl<LoaderResult, FromDrop: LiquidDrop + 'static, ToDrop, T>
  LoaderResultToDropsFn<LoaderResult, FromDrop, ToDrop> for T
where
  T: Fn(Option<LoaderResult>, DropRef<FromDrop>) -> Result<Vec<ToDrop>, DropError> + Send + Sync,
{
}

pub trait DropsToValueFn<Drop: LiquidDrop + 'static, Value: ValueView + Clone>
where
  Self: Fn(&DropStore<DropStoreID<Drop>>, Vec<DropRef<Drop>>) -> Result<DropResult<Value>, DropError>
    + Send
    + Sync,
{
}

impl<Drop: LiquidDrop + 'static, Value: ValueView + Clone, T> DropsToValueFn<Drop, Value> for T where
  T: Fn(&DropStore<DropStoreID<Drop>>, Vec<DropRef<Drop>>) -> Result<DropResult<Value>, DropError>
    + Send
    + Sync
{
}

pub trait GetOnceCellFn<Drop: LiquidDrop, Value: ValueView + Clone>
where
  Self: for<'a> Fn(&'a <Drop as LiquidDrop>::Cache) -> &'a OnceBox<DropResult<Value>> + Send + Sync,
{
}

impl<Drop: LiquidDrop, Value: ValueView + Clone, T> GetOnceCellFn<Drop, Value> for T where
  T: for<'a> Fn(&'a <Drop as LiquidDrop>::Cache) -> &'a OnceBox<DropResult<Value>> + Send + Sync
{
}

pub trait GetInverseOnceCellFn<FromDrop: LiquidDrop, ToDrop: LiquidDrop>
where
  Self: Fn(&ToDrop::Cache) -> &OnceBox<DropResult<FromDrop>> + Send + Sync,
{
}

impl<FromDrop: LiquidDrop, ToDrop: LiquidDrop, T> GetInverseOnceCellFn<FromDrop, ToDrop> for T where
  T: Fn(&ToDrop::Cache) -> &OnceBox<DropResult<FromDrop>> + Send + Sync
{
}

pub trait PreloaderBuilder {
  type Preloader;
  type Error;

  fn finalize(self) -> Result<Self::Preloader, Self::Error>;
}

#[async_trait]
pub trait Preloader<
  FromDrop: Send + Sync + LiquidDrop<Context = Context> + Clone + Into<DropResult<FromDrop>>,
  ToDrop: Send + Sync + LiquidDrop<Context = Context> + Clone + 'static,
  V: ValueView + Clone + Send + Sync + DropResultTrait<V> + 'static,
  Context: crate::Context,
>
{
  type LoaderResult;

  fn loader_result_to_drops(
    &self,
    result: Option<Self::LoaderResult>,
    drop: DropRef<FromDrop>,
  ) -> Result<Vec<ToDrop>, DropError>;
  fn with_drop_store<'store, R: 'store, F: FnOnce(&DropStore<Context::StoreID>) -> R>(
    &'store self,
    f: F,
  ) -> R;
  fn drops_to_value(
    &self,
    store: &DropStore<Context::StoreID>,
    drops: Vec<DropRef<ToDrop>>,
  ) -> Result<DropResult<V>, DropError>;
  fn get_once_cell<'a>(&self, cache: &'a FromDrop::Cache) -> &'a OnceBox<DropResult<V>>;
  fn get_inverse_once_cell<'a>(
    &self,
    cache: &'a ToDrop::Cache,
  ) -> Option<&'a OnceBox<DropResult<FromDrop>>>;
  async fn load_model_lists(
    &self,
    db: &ConnectionWrapper,
    ids: &[Context::StoreID],
  ) -> Result<HashMap<Context::StoreID, Self::LoaderResult>, DropError>;

  async fn preload(
    &self,
    db: &ConnectionWrapper,
    drops: &[DropRef<FromDrop>],
  ) -> Result<PreloaderResult<Context::StoreID, V>, DropError>
  where
    FromDrop: 'static,
    Context::StoreID: From<FromDrop::ID> + From<ToDrop::ID>,
  {
    let span = info_span!(
      "preload",
      from_drop = std::any::type_name::<FromDrop>(),
      to_drop = std::any::type_name::<ToDrop>(),
    );

    let _enter = span.enter();

    let preloader_result = self
      .with_drop_store(|store| {
        let loaded_values_by_drop_id: HashMap<_, _> = drops
          .iter()
          .cloned()
          .filter_map(|drop| {
            let cache = store.get_drop_cache::<FromDrop>(drop.id());
            let once_cell = self.get_once_cell(&cache);
            once_cell.get().map(|value| (drop.id(), value.clone()))
          })
          .collect();

        let unloaded_drops = drops
          .iter()
          .cloned()
          .filter(|drop| !loaded_values_by_drop_id.contains_key(&drop.id()));

        let unloaded_drops_by_id = unloaded_drops
          .map(|drop| (drop.id(), drop))
          .collect::<HashMap<_, _>>();

        async move {
          let newly_loaded_drop_lists = self.load_drop_lists(&unloaded_drops_by_id, db).await?;

          self.populate_db_preloader_results(
            unloaded_drops_by_id,
            newly_loaded_drop_lists,
            loaded_values_by_drop_id,
          )
        }
      })
      .await?;

    Ok(preloader_result)
  }

  async fn load_drop_lists(
    &self,
    unloaded_drops_by_id: &HashMap<Context::StoreID, DropRef<FromDrop>>,
    db: &ConnectionWrapper,
  ) -> Result<HashMap<Context::StoreID, Vec<DropRef<ToDrop>>>, DropError>
  where
    FromDrop: 'static,
    Context::StoreID: From<ToDrop::ID>,
  {
    let span = info_span!(
      "load_drop_lists",
      from_drop = std::any::type_name::<FromDrop>(),
      to_drop = std::any::type_name::<ToDrop>(),
      ids = unloaded_drops_by_id
        .keys()
        .map(|key| format!("{}", key))
        .collect::<Vec<_>>()
        .join(", ")
    );

    let _enter = span.enter();

    let mut model_lists = self
      .load_model_lists(
        db,
        unloaded_drops_by_id
          .keys()
          .copied()
          .collect::<Vec<_>>()
          .as_slice(),
      )
      .await?;

    let newly_loaded_drop_lists = self.with_drop_store(|store| {
      unloaded_drops_by_id
        .iter()
        .map(|(id, drop)| {
          let result = model_lists.remove(id);
          let converted_drops = self.loader_result_to_drops(result, drop.clone())?;
          let normalized_drops: Vec<DropRef<ToDrop>> = store.store_all(converted_drops);
          Ok((*id, normalized_drops))
        })
        .collect::<Result<HashMap<_, _>, DropError>>()
    })?;

    for (id, drops) in newly_loaded_drop_lists.iter() {
      let from_drop = unloaded_drops_by_id.get(id).unwrap();
      self.populate_inverse_caches(from_drop.clone(), drops)
    }

    Ok(newly_loaded_drop_lists)
  }

  fn populate_db_preloader_results(
    &self,
    drops_by_id: HashMap<Context::StoreID, DropRef<FromDrop>>,
    newly_loaded_drop_lists: HashMap<Context::StoreID, Vec<DropRef<ToDrop>>>,
    loaded_values_by_drop_id: HashMap<Context::StoreID, DropResult<V>>,
  ) -> Result<PreloaderResult<Context::StoreID, V>, DropError>
  where
    Context::StoreID: From<FromDrop::ID>,
  {
    let mut values_by_id: HashMap<Context::StoreID, DropResult<V>> =
      HashMap::with_capacity(newly_loaded_drop_lists.len() + loaded_values_by_drop_id.len());

    self.with_drop_store(|store| {
      newly_loaded_drop_lists
        .into_iter()
        .try_for_each(|(id, drop_list)| {
          let value = (self.drops_to_value(store, drop_list) as Result<DropResult<V>, DropError>)?;

          values_by_id.insert(id, value.clone());

          let drop = drops_by_id.get(&id).unwrap().fetch();
          let cache = store.get_drop_cache::<FromDrop>(drop.id().into());
          let once_cell = self.get_once_cell(&cache);
          once_cell.get_or_init(|| Box::new(value));
          Ok::<_, DropError>(())
        })
    })?;

    for (id, value) in loaded_values_by_drop_id {
      values_by_id.insert(id, value.clone());
    }

    Ok(PreloaderResult::new(values_by_id))
  }

  fn populate_inverse_caches(&self, from_drop: DropRef<FromDrop>, drops: &Vec<DropRef<ToDrop>>)
  where
    FromDrop: 'static,
    Context::StoreID: From<ToDrop::ID>,
  {
    self.with_drop_store(|store| {
      for to_drop in drops {
        let to_drop = to_drop.fetch();
        let cache = store.get_drop_cache::<ToDrop>(to_drop.id().into());
        let inverse_once_cell = self.get_inverse_once_cell(&cache);
        if let Some(inverse_once_cell) = inverse_once_cell {
          inverse_once_cell.get_or_init(|| Box::new(from_drop.clone().into()));
        }
      }
    });
  }

  async fn load_single(
    &self,
    db: &ConnectionWrapper,
    drop: DropRef<FromDrop>,
  ) -> Result<DropResult<V>, DropError>
  where
    FromDrop: 'static,
    Context::StoreID: From<FromDrop::ID> + From<ToDrop::ID>,
  {
    let span = warn_span!(
      "load_single",
      from_drop = std::any::type_name::<FromDrop>(),
      to_drop = std::any::type_name::<ToDrop>(),
      id = format!("{}", drop.id())
    );

    let _enter = span.enter();

    warn!(
      "N+1 query detected for {} {} -> {}",
      std::any::type_name::<FromDrop>(),
      drop.id(),
      std::any::type_name::<V>()
    );

    let preloader_result = self.preload(db, &[drop.clone()]).await?;
    let single_value = preloader_result.get(drop.id());

    Ok(single_value)
  }

  async fn expect_single(
    &self,
    db: &ConnectionWrapper,
    drop: DropRef<FromDrop>,
  ) -> Result<V, DropError>
  where
    FromDrop: 'static,
    Context::StoreID: From<FromDrop::ID> + From<ToDrop::ID>,
  {
    let loaded = self.load_single(db, drop).await?;
    let inner = loaded.get_inner_cloned();

    inner.ok_or_else(|| DropError::ExpectedEntityNotFound(std::any::type_name::<V>().to_string()))
  }
}
