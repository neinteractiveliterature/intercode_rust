use std::{
  collections::HashMap,
  error::Error,
  fmt::{Debug, Display},
  hash::Hash,
  pin::Pin,
};

use async_trait::async_trait;
use lazy_liquid_value_view::{DropResult, LiquidDrop};
use liquid::ValueView;
use once_cell::race::OnceBox;
use sea_orm::DatabaseConnection;
use tracing::warn;

use crate::DropError;

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

  #[allow(dead_code)]
  pub fn all_values_unwrapped(&self) -> Vec<Value> {
    self
      .all_values()
      .into_iter()
      .map(|value| value.get_inner().unwrap().clone())
      .collect::<Vec<_>>()
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

pub trait GetValueFn<'a, LoaderResult: 'a, Value, Drop: 'a>
where
  Self: Fn(Option<&'a LoaderResult>, &'a Drop) -> Result<Option<Value>, DropError> + Send + Sync,
{
}

impl<'a, LoaderResult: 'a, Value, Drop: 'a, T> GetValueFn<'a, LoaderResult, Value, Drop> for T where
  T: Fn(Option<&'a LoaderResult>, &'a Drop) -> Result<Option<Value>, DropError> + Send + Sync
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
pub trait Preloader<Drop: Send + Sync + Debug, Id: Eq + Hash, V: ValueView + Clone> {
  fn get_id(&self, drop: &Drop) -> Id;
  async fn preload(
    &self,
    db: &DatabaseConnection,
    drops: &[&Drop],
  ) -> Result<PreloaderResult<Id, V>, DropError>;

  async fn load_single(
    &self,
    db: &DatabaseConnection,
    drop: &Drop,
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

  async fn expect_single(&self, db: &DatabaseConnection, drop: &Drop) -> Result<V, DropError> {
    self
      .load_single(db, drop)
      .await?
      .ok_or_else(|| DropError::ExpectedEntityNotFound(std::any::type_name::<V>().to_string()))
  }
}

pub async fn populate_db_preloader_results<
  ID: Eq + std::hash::Hash + Clone + From<i64> + Send + Sync,
  Drop: LiquidDrop + Send + Sync,
  Value: ValueView + Into<DropResult<Value>> + Clone + Sync + Send,
  LoaderResult,
>(
  drops: &[&Drop],
  model_lists: HashMap<ID, LoaderResult>,
  get_id: &Pin<Box<dyn for<'a> GetIdFn<'a, ID, Drop>>>,
  get_value: &Pin<Box<dyn for<'a> GetValueFn<'a, LoaderResult, Value, Drop>>>,
  get_once_cell: &Pin<Box<dyn for<'a> GetOnceCellFn<'a, Drop, Value>>>,
) -> Result<PreloaderResult<ID, Value>, DropError> {
  let mut values_by_id: HashMap<ID, DropResult<Value>> = HashMap::with_capacity(model_lists.len());

  for drop in drops {
    let id = get_id(drop);
    let result = model_lists.get(&id);
    let value = (get_value)(result, *drop)?;
    let drop_result: DropResult<Value> = value.into();

    values_by_id.insert(id, drop_result.clone());

    let once_cell = (get_once_cell)(drop.get_cache());
    once_cell.get_or_init(|| Box::new(drop_result));
  }

  Ok(PreloaderResult { values_by_id })
}

pub fn populate_inverse_caches<
  ID: Eq + std::hash::Hash + Clone + From<i64>,
  FromDrop: LiquidDrop + Clone,
  Value: ValueView + Into<DropResult<Value>> + Clone,
>(
  from_drop: &FromDrop,
  preloader_result: &PreloaderResult<ID, Value>,
  get_inverse_once_cell: &Pin<Box<dyn for<'a> GetInverseOnceCellFn<'a, FromDrop, Value>>>,
) {
  let from_drop_result: DropResult<FromDrop> = from_drop.into();

  for to_drop_result in preloader_result.values_by_id.values() {
    let to_drop = to_drop_result.get_inner();
    if let Some(to_drop) = to_drop {
      let inverse_once_cell = (get_inverse_once_cell)(to_drop);
      inverse_once_cell.get_or_init(|| Box::new(from_drop_result.clone()));
    }
  }
}
