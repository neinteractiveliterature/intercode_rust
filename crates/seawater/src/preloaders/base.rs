use std::{
  collections::HashMap,
  error::Error,
  fmt::{Debug, Display},
  hash::Hash,
};

use crate::{optional_value_view::OptionalValueView, DropResult, DropResultTrait, LiquidDrop};
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
  Self: Fn(Vec<DropRef<Drop>>) -> Result<DropResult<Value>, DropError> + Send + Sync,
{
}

impl<Drop: LiquidDrop + 'static, Value: ValueView + Clone, T> DropsToValueFn<Drop, Value> for T where
  T: Fn(Vec<DropRef<Drop>>) -> Result<DropResult<Value>, DropError> + Send + Sync
{
}

pub trait GetOnceCellFn<'a, Drop: LiquidDrop, Value: ValueView + Clone + 'a>
where
  Self: Fn(&'a <Drop as LiquidDrop>::Cache) -> &'a OnceBox<DropResult<Value>> + Send + Sync,
  <Drop as LiquidDrop>::Cache: 'a,
{
}

impl<'a, Drop: LiquidDrop, Value: ValueView + Clone + 'a, T> GetOnceCellFn<'a, Drop, Value> for T
where
  T: Fn(&'a <Drop as LiquidDrop>::Cache) -> &'a OnceBox<DropResult<Value>> + Send + Sync,
  <Drop as LiquidDrop>::Cache: 'a,
{
}

pub trait GetInverseOnceCellFn<'a, FromDrop: LiquidDrop, ToDrop: LiquidDrop + 'a>
where
  Self: Fn(&'a ToDrop::Cache) -> &'a OnceBox<DropResult<FromDrop>> + Send + Sync,
  FromDrop: 'a,
{
}

impl<'a, FromDrop: LiquidDrop, ToDrop: LiquidDrop + 'a, T>
  GetInverseOnceCellFn<'a, FromDrop, ToDrop> for T
where
  T: Fn(&'a ToDrop::Cache) -> &'a OnceBox<DropResult<FromDrop>> + Send + Sync,
  FromDrop: 'a,
{
}

pub trait PreloaderBuilder {
  type Preloader;
  type Error;

  fn finalize(self) -> Result<Self::Preloader, Self::Error>;
}

#[async_trait]
pub trait Preloader<
  FromDrop: Send + Sync + LiquidDrop<ID = Id> + Clone + Into<DropResult<FromDrop>>,
  ToDrop: Send + Sync + LiquidDrop<ID = Id> + Clone + 'static,
  Id: Eq + Hash + Copy + Send + Sync + Display + Debug + 'static,
  V: ValueView + Clone + Send + Sync + DropResultTrait<V> + 'static,
> where
  i64: From<<ToDrop as LiquidDrop>::ID>,
{
  type LoaderResult;

  fn loader_result_to_drops(
    &self,
    result: Option<Self::LoaderResult>,
    drop: DropRef<FromDrop>,
  ) -> Result<Vec<ToDrop>, DropError>;
  fn with_drop_store<'store, R, F: FnOnce(&'store DropStore<Id>) -> R>(&self, f: F) -> R;
  fn drops_to_value(&self, drops: Vec<DropRef<ToDrop>>) -> Result<DropResult<V>, DropError>;
  fn get_once_cell<'a>(&'a self, cache: &'a FromDrop::Cache) -> &'a OnceBox<DropResult<V>>;
  fn get_inverse_once_cell<'a>(
    &'a self,
    cache: &'a ToDrop::Cache,
  ) -> Option<&'a OnceBox<DropResult<FromDrop>>>;
  async fn load_model_lists(
    &self,
    db: &ConnectionWrapper,
    ids: &[Id],
  ) -> Result<HashMap<Id, Self::LoaderResult>, DropError>;

  async fn preload(
    &self,
    db: &ConnectionWrapper,
    drops: &[DropRef<FromDrop>],
  ) -> Result<PreloaderResult<Id, V>, DropError>
  where
    i64: From<FromDrop::ID>,
    Id: From<FromDrop::ID>,
    FromDrop: 'static,
  {
    let span = info_span!(
      "preload",
      from_drop = std::any::type_name::<FromDrop>(),
      to_drop = std::any::type_name::<ToDrop>(),
    );

    let _enter = span.enter();

    let loaded_values_by_drop_id: HashMap<Id, DropResult<V>> = self.with_drop_store(|store| {
      drops
        .iter()
        .cloned()
        .filter_map(|drop| {
          let cache = store.get_drop_cache::<FromDrop>(drop.id());
          let once_cell = self.get_once_cell(&cache);
          once_cell.get().map(|value| (drop.id(), value.clone()))
        })
        .collect()
    });

    let unloaded_drops = drops
      .iter()
      .cloned()
      .filter(|drop| !loaded_values_by_drop_id.contains_key(&drop.id()));

    let unloaded_drops_by_id = unloaded_drops
      .map(|drop| (drop.id(), drop))
      .collect::<HashMap<_, _>>();

    let newly_loaded_drop_lists = self.load_drop_lists(&unloaded_drops_by_id, db).await?;

    let preloader_result = self.populate_db_preloader_results(
      unloaded_drops_by_id,
      newly_loaded_drop_lists,
      loaded_values_by_drop_id,
    )?;

    Ok(preloader_result)
  }

  async fn load_drop_lists(
    &self,
    unloaded_drops_by_id: &HashMap<Id, DropRef<FromDrop>>,
    db: &ConnectionWrapper,
  ) -> Result<HashMap<Id, Vec<DropRef<ToDrop>>>, DropError>
  where
    i64: From<FromDrop::ID>,
    FromDrop: 'static,
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

    let newly_loaded_drop_lists = self.with_drop_store(|normalized_drop_cache| {
      unloaded_drops_by_id
        .iter()
        .map(|(id, drop)| {
          let result = model_lists.remove(id);
          let converted_drops = self.loader_result_to_drops(result, drop.clone())?;
          let normalized_drops: Vec<DropRef<ToDrop, Id>> =
            normalized_drop_cache.store_all(converted_drops);
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
    drops_by_id: HashMap<Id, DropRef<FromDrop>>,
    newly_loaded_drop_lists: HashMap<Id, Vec<DropRef<ToDrop>>>,
    loaded_values_by_drop_id: HashMap<Id, DropResult<V>>,
  ) -> Result<PreloaderResult<Id, V>, DropError>
  where
    i64: From<FromDrop::ID>,
  {
    let mut values_by_id: HashMap<Id, DropResult<V>> =
      HashMap::with_capacity(newly_loaded_drop_lists.len() + loaded_values_by_drop_id.len());

    self.with_drop_store(|store| {
      newly_loaded_drop_lists
        .into_iter()
        .try_for_each(|(id, drop_list)| {
          let value = (self.drops_to_value(drop_list) as Result<DropResult<V>, DropError>)?;

          values_by_id.insert(id, value.clone());

          let drop = drops_by_id.get(&id).unwrap().fetch();
          let cache = store.get_drop_cache::<FromDrop>(drop.id());
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

  fn populate_inverse_caches<'store>(
    &self,
    from_drop: DropRef<'store, FromDrop>,
    drops: &Vec<DropRef<'store, ToDrop>>,
  ) where
    FromDrop: 'static,
  {
    self.with_drop_store(|store| {
      for to_drop in drops {
        let to_drop = to_drop.fetch();
        let cache = store.get_drop_cache::<ToDrop>(to_drop.id());
        let inverse_once_cell = self.get_inverse_once_cell(&cache);
        if let Some(inverse_once_cell) = inverse_once_cell {
          inverse_once_cell
            .get_or_init(|| Box::new(DropResult::new(DropRef::new(from_drop.id(), store))));
        }
      }
    });
  }

  async fn load_single(
    &self,
    db: &ConnectionWrapper,
    drop: DropRef<'async_trait, FromDrop>,
  ) -> Result<DropResult<OptionalValueView<V>>, DropError>
  where
    i64: From<FromDrop::ID>,
    FromDrop: 'static,
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

    Ok(self.preload(db, &[drop.clone()]).await?.get(&drop.id()))
  }

  async fn expect_single(
    &self,
    db: &ConnectionWrapper,
    drop: DropRef<'async_trait, FromDrop>,
  ) -> Result<V, DropError>
  where
    i64: From<FromDrop::ID>,
    FromDrop: 'static,
  {
    let loaded = self.load_single(db, drop).await?;

    loaded
      .get_inner()
      .as_option()
      .cloned()
      .ok_or_else(|| DropError::ExpectedEntityNotFound(std::any::type_name::<V>().to_string()))
  }
}
