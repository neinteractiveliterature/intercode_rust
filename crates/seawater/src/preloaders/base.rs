use std::{
  collections::{hash_map::IterMut, HashMap},
  error::Error,
  fmt::{Debug, Display},
  hash::Hash,
  sync::Arc,
};

use async_trait::async_trait;
use lazy_liquid_value_view::{DropResult, LiquidDrop, LiquidDropWithID};
use liquid::ValueView;
use once_cell::race::OnceBox;
use sea_orm::DatabaseConnection;
use tracing::warn;

use crate::{DropError, NormalizedDropCache};

#[derive(Debug, Clone)]
pub struct IncompleteBuilderError;

impl Display for IncompleteBuilderError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Builder parameters incomplete")
  }
}

impl Error for IncompleteBuilderError {}

#[derive(Debug)]
pub struct PreloaderResult<Id: Eq + Hash, Value: ValueView + Clone> {
  values_by_id: HashMap<Id, DropResult<Value>>,
}

impl<Id: Eq + Hash, Value: ValueView + Clone> PreloaderResult<Id, Value> {
  pub fn get(&self, id: &Id) -> DropResult<Value> {
    self.values_by_id.get(id).cloned().unwrap_or_default()
  }

  #[allow(dead_code)]
  pub fn expect_value(&self, id: &Id) -> Result<Value, DropError> {
    self
      .get(id)
      .get_inner()
      .cloned()
      .ok_or_else(|| DropError::ExpectedEntityNotFound(std::any::type_name::<Value>().to_string()))
  }

  pub fn all_values(&self) -> Vec<DropResult<Value>> {
    self.values_by_id.values().cloned().collect::<Vec<_>>()
  }

  pub fn iter_mut(&mut self) -> IterMut<Id, DropResult<Value>> {
    self.values_by_id.iter_mut()
  }

  #[allow(dead_code)]
  pub fn all_values_unwrapped(&self) -> Vec<Value> {
    self
      .all_values()
      .into_iter()
      .map(|value| value.get_inner().unwrap().clone())
      .collect::<Vec<_>>()
  }

  pub fn extend(&mut self, items: &mut dyn Iterator<Item = (Id, DropResult<Value>)>) {
    self.values_by_id.extend(items)
  }
}

impl<Id: Eq + Hash, Value: ValueView + Clone> PreloaderResult<Id, Vec<Value>> {
  pub fn all_values_flat(&self) -> Vec<DropResult<Value>> {
    self
      .all_values()
      .iter()
      .flat_map(|drop_result| drop_result.get_inner().unwrap())
      .map(|value| value.into())
      .collect::<Vec<_>>()
  }

  pub fn all_values_flat_unwrapped(&self) -> Vec<Value> {
    self
      .all_values_flat()
      .into_iter()
      .map(|value| value.get_inner().unwrap().clone())
      .collect::<Vec<_>>()
  }
}

pub trait GetIdFn<'a, ID, Drop: 'a>
where
  Self: Fn(&'a Drop) -> ID + Send + Sync,
{
}

impl<'a, ID, Drop: 'a, T> GetIdFn<'a, ID, Drop> for T where T: Fn(&'a Drop) -> ID + Send + Sync {}

pub trait LoaderResultToDropsFn<'a, LoaderResult: 'a, FromDrop: 'a, ToDrop>
where
  Self: Fn(Option<&'a LoaderResult>, &'a FromDrop) -> Result<Vec<ToDrop>, DropError> + Send + Sync,
{
}

impl<'a, LoaderResult: 'a, FromDrop: 'a, ToDrop, T>
  LoaderResultToDropsFn<'a, LoaderResult, FromDrop, ToDrop> for T
where
  T: Fn(Option<&'a LoaderResult>, &'a FromDrop) -> Result<Vec<ToDrop>, DropError> + Send + Sync,
{
}

pub trait DropsToValueFn<Drop, Value: ValueView>
where
  Self: Fn(Vec<Arc<Drop>>) -> Result<DropResult<Value>, DropError> + Send + Sync,
{
}

impl<Drop, Value: ValueView, T> DropsToValueFn<Drop, Value> for T where
  T: Fn(Vec<Arc<Drop>>) -> Result<DropResult<Value>, DropError> + Send + Sync
{
}

pub trait GetOnceCellFn<'a, Drop: LiquidDrop, Value: ValueView + 'a>
where
  Self: Fn(&'a <Drop as LiquidDrop>::Cache) -> &'a OnceBox<DropResult<Value>> + Send + Sync,
  <Drop as LiquidDrop>::Cache: 'a,
{
}

impl<'a, Drop: LiquidDrop, Value: ValueView + 'a, T> GetOnceCellFn<'a, Drop, Value> for T
where
  T: Fn(&'a <Drop as LiquidDrop>::Cache) -> &'a OnceBox<DropResult<Value>> + Send + Sync,
  <Drop as LiquidDrop>::Cache: 'a,
{
}

pub trait GetInverseOnceCellFn<'a, FromDrop: LiquidDrop, Value: ValueView + 'a>
where
  Self: Fn(&'a Value) -> &'a OnceBox<DropResult<FromDrop>> + Send + Sync,
  FromDrop: 'a,
{
}

impl<'a, FromDrop: LiquidDrop, Value: ValueView + 'a, T> GetInverseOnceCellFn<'a, FromDrop, Value>
  for T
where
  T: Fn(&'a Value) -> &'a OnceBox<DropResult<FromDrop>> + Send + Sync,
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
  FromDrop: Send + Sync + LiquidDrop + Clone + Into<DropResult<FromDrop>>,
  ToDrop: Send + Sync + LiquidDrop + LiquidDropWithID + 'static,
  Id: Eq + Hash + Copy + Send + Sync,
  V: ValueView + Clone + Send + Sync,
> where
  i64: From<<ToDrop as LiquidDropWithID>::ID>,
{
  type LoaderResult;

  fn get_id(&self, drop: &FromDrop) -> Id;
  fn loader_result_to_drops(
    &self,
    result: Option<&Self::LoaderResult>,
    drop: &FromDrop,
  ) -> Result<Vec<ToDrop>, DropError>;
  fn get_normalized_drop_cache(&self) -> Option<&NormalizedDropCache<i64>>;
  fn drops_to_value(&self, drops: Vec<Arc<ToDrop>>) -> Result<DropResult<V>, DropError>;
  fn get_once_cell<'a>(&'a self, cache: &'a FromDrop::Cache) -> &'a OnceBox<DropResult<V>>;
  fn get_inverse_once_cell<'a>(
    &'a self,
    drop: &'a ToDrop,
  ) -> Option<&'a OnceBox<DropResult<FromDrop>>>;
  async fn load_model_lists(
    &self,
    db: &DatabaseConnection,
    ids: &[Id],
  ) -> Result<HashMap<Id, Self::LoaderResult>, DropError>;

  async fn preload(
    &self,
    db: &DatabaseConnection,
    drops: &[&FromDrop],
  ) -> Result<PreloaderResult<Id, V>, DropError> {
    let drops_by_id = drops
      .iter()
      .map(|drop| (self.get_id(drop), *drop))
      .collect::<HashMap<_, _>>();

    let model_lists = self
      .load_model_lists(
        db,
        drops_by_id.keys().copied().collect::<Vec<_>>().as_slice(),
      )
      .await?;

    let drop_lists = drops_by_id
      .iter()
      .map(|(id, drop)| {
        let result = model_lists.get(id);
        let converted_drops = self.loader_result_to_drops(result, drop)?;
        let normalized_drops: Vec<Arc<ToDrop>> =
          if let Some(normalized_drop_cache) = self.get_normalized_drop_cache() {
            converted_drops
              .into_iter()
              .map(|drop| {
                let cached = normalized_drop_cache.get(i64::from(drop.id()))?;
                if let Some(cached) = cached {
                  Ok::<Arc<ToDrop>, DropError>(cached)
                } else {
                  Ok(normalized_drop_cache.put(drop)?)
                }
              })
              .collect()
          } else {
            Ok(converted_drops.into_iter().map(Arc::new).collect())
          }?;
        Ok((*id, normalized_drops))
      })
      .collect::<Result<HashMap<_, _>, DropError>>()?;

    for (id, drops) in drop_lists.iter() {
      let from_drop = drops_by_id.get(id).unwrap();
      self.populate_inverse_caches(*from_drop, drops)
    }

    let preloader_result = self.populate_db_preloader_results(drops_by_id, drop_lists)?;

    Ok(preloader_result)
  }

  fn populate_db_preloader_results(
    &self,
    drops_by_id: HashMap<Id, &FromDrop>,
    drop_lists: HashMap<Id, Vec<Arc<ToDrop>>>,
  ) -> Result<PreloaderResult<Id, V>, DropError> {
    let mut values_by_id: HashMap<Id, DropResult<V>> = HashMap::with_capacity(drop_lists.len());

    for (id, drop_list) in drop_lists {
      let value = (self.drops_to_value(drop_list) as Result<DropResult<V>, DropError>)?;

      values_by_id.insert(id, value.clone());

      let drop = drops_by_id.get(&id).unwrap();
      let once_cell = self.get_once_cell(drop.get_cache());
      once_cell.get_or_init(|| Box::new(value));
    }

    Ok(PreloaderResult { values_by_id })
  }

  fn populate_inverse_caches(&self, from_drop: &FromDrop, drops: &Vec<Arc<ToDrop>>) {
    let from_drop_result: DropResult<FromDrop> = from_drop.into();

    for to_drop in drops {
      let inverse_once_cell = self.get_inverse_once_cell(to_drop.as_ref());
      if let Some(inverse_once_cell) = inverse_once_cell {
        inverse_once_cell.get_or_init(|| Box::new(from_drop_result.clone()));
      }
    }
  }

  async fn load_single(
    &self,
    db: &DatabaseConnection,
    drop: &FromDrop,
  ) -> Result<Option<V>, DropError> {
    warn!(
      "N+1 query detected for {:?} -> {}",
      drop,
      std::any::type_name::<V>()
    );

    Ok(
      self
        .preload(db, &[drop])
        .await?
        .get(&self.get_id(drop))
        .get_inner()
        .cloned(),
    )
  }

  async fn expect_single(&self, db: &DatabaseConnection, drop: &FromDrop) -> Result<V, DropError> {
    self
      .load_single(db, drop)
      .await?
      .ok_or_else(|| DropError::ExpectedEntityNotFound(std::any::type_name::<V>().to_string()))
  }
}
