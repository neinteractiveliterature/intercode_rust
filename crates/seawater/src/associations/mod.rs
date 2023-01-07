mod association_type_adapters;
mod elided_model;
mod registry;
mod value_adapters;

use std::{fmt::Debug, future::Future, hash::Hash, pin::Pin};

use async_graphql::futures_util::future::try_join_all;
use lazy_liquid_value_view::{DropResult, LiquidDrop, LiquidDropWithID};
use liquid::ValueView;
use once_cell::race::OnceBox;
use sea_orm::{EntityTrait, Linked, ModelTrait, PrimaryKeyTrait};
use tracing::info;

use crate::{
  loaders::AssociationLoaderResult, preloaders::PreloaderResult, pretty_type_name, Context,
  ContextContainer, DropEntity, DropError, DropPrimaryKeyValue, ModelBackedDrop,
};

use self::{
  association_type_adapters::ModelBackedDropAssociationAssociationTypeAdapter,
  registry::{BoxAssociation, ASSOCIATION_REGISTRY},
  value_adapters::ModelBackedDropAssociationValueAdapter,
};

pub enum AssociationType<FromDrop: ModelBackedDrop, ToDrop: ModelBackedDrop> {
  Related,
  Linked(
    Box<dyn Linked<FromEntity = DropEntity<FromDrop>, ToEntity = DropEntity<ToDrop>> + Send + Sync>,
  ),
}

impl<FromDrop: ModelBackedDrop, ToDrop: ModelBackedDrop> Debug
  for AssociationType<FromDrop, ToDrop>
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Related => write!(f, "Related"),
      Self::Linked(arg0) => f.debug_tuple("Linked").finish(),
    }
  }
}

#[derive(Debug)]
pub enum TargetType {
  OneOptional,
  OneRequired,
  Many,
}

#[derive(Debug, Clone)]
pub struct EagerLoadInstruction {
  association_name: String,
  children: Vec<EagerLoadInstruction>,
}

#[derive(Debug)]
pub struct AssociationMeta<FromDrop: ModelBackedDrop, ToDrop: ModelBackedDrop> {
  association_type: AssociationType<FromDrop, ToDrop>,
  target_type: TargetType,
  field_name: String,
  inverse_field_name: Option<String>,
  name: String,
  eager_loads: Vec<EagerLoadInstruction>,
}

pub trait ModelBackedDropAssociation {
  type FromDrop: ModelBackedDrop + ContextContainer;
  type ToDrop: ModelBackedDrop;
  type Value: ValueView + Clone;
  type Preloader;
  type PrimaryKeyValue: Eq + Hash;

  fn loader_result_to_drops(
    &self,
    result: Option<
      &dyn AssociationLoaderResult<
        <Self::FromDrop as ModelBackedDrop>::Model,
        <Self::ToDrop as ModelBackedDrop>::Model,
      >,
    >,
    from_drop: &Self::FromDrop,
  ) -> Result<Vec<Self::ToDrop>, DropError>;

  fn association_type_adapter(
    &self,
  ) -> &dyn ModelBackedDropAssociationAssociationTypeAdapter<
    FromDrop = Self::FromDrop,
    ToDrop = Self::ToDrop,
    Preloader = Self::Preloader,
    Value = Self::Value,
  >;

  fn value_adapter(
    &self,
  ) -> &dyn ModelBackedDropAssociationValueAdapter<
    Id = DropPrimaryKeyValue<Self::FromDrop>,
    ToDrop = Self::ToDrop,
    Value = Self::Value,
  >;

  fn get_once_cell(
    &self,
    cache: &<Self::FromDrop as LiquidDrop>::Cache,
  ) -> &OnceBox<DropResult<Self::Value>>;

  fn preload<'a, 'life0, 'async_trait>(
    &'life0 self,
    context: <Self::FromDrop as ContextContainer>::Context,
    drops: &'a [&'a Self::FromDrop],
  ) -> Pin<
    Box<
      dyn Future<Output = Result<PreloaderResult<Self::PrimaryKeyValue, Self::Value>, DropError>>
        + Send
        + 'async_trait,
    >,
  >
  where
    'a: 'async_trait,
    'life0: 'async_trait,
    Self: 'async_trait;

  fn get_meta(&self) -> &AssociationMeta<Self::FromDrop, Self::ToDrop>;
}

pub trait AssociationContainer<ToDrop, Preloader, Value>
where
  Self: ModelBackedDrop,
{
  fn get_association(
    name: &str,
  ) -> Option<
    &Box<
      dyn ModelBackedDropAssociation<
        FromDrop = Self,
        ToDrop = ToDrop,
        Preloader = Preloader,
        Value = Value,
        PrimaryKeyValue = DropPrimaryKeyValue<Self>,
      >,
    >,
  >;
}

#[derive(Debug)]
pub struct Association<
  FromDrop: ModelBackedDrop + LiquidDropWithID + Send + Sync + Clone,
  ToDrop: ModelBackedDrop + LiquidDropWithID + Send + Sync + 'static,
  Preloader: crate::preloaders::Preloader<FromDrop, ToDrop, DropPrimaryKeyValue<FromDrop>, Value>,
  Value: ValueView + Clone + Send + Sync,
> where
  FromDrop: Into<DropResult<FromDrop>>,
  DropPrimaryKeyValue<FromDrop>: Sync + Send + Eq + Clone + Hash + From<i64> + Copy,
  ToDrop::ID: From<i64>,
  i64: From<ToDrop::ID>,
{
  value_adapter: Box<
    dyn ModelBackedDropAssociationValueAdapter<
      Id = DropPrimaryKeyValue<FromDrop>,
      ToDrop = ToDrop,
      Value = Value,
    >,
  >,
  association_type_adapter: Box<
    dyn ModelBackedDropAssociationAssociationTypeAdapter<
      FromDrop = FromDrop,
      ToDrop = ToDrop,
      Preloader = Preloader,
      Value = Value,
    >,
  >,
  meta: AssociationMeta<FromDrop, ToDrop>,
}

impl<
    C: Context,
    FromDrop: ModelBackedDrop<Context = C> + LiquidDropWithID + Send + Sync + Clone,
    ToDrop: ModelBackedDrop<Context = C> + LiquidDropWithID + Send + Sync + 'static,
    Preloader: crate::preloaders::Preloader<FromDrop, ToDrop, DropPrimaryKeyValue<FromDrop>, Value> + Sync,
    Value: ValueView + Clone + Send + Sync,
  > Association<FromDrop, ToDrop, Preloader, Value>
where
  FromDrop: Send + Sync + LiquidDropWithID + Into<DropResult<FromDrop>>,
  ToDrop: Send + Sync,
  DropPrimaryKeyValue<FromDrop>: Sync + Send + Eq + Clone + Hash + From<i64> + Copy,
  DropPrimaryKeyValue<ToDrop>: Sync + Send + Eq + Clone + Hash + From<i64> + Copy,
  ToDrop::ID: From<i64>,
  i64: From<ToDrop::ID>,
{
  pub async fn preload<'a>(
    &self,
        context: <FromDrop as ContextContainer>::Context,
        drops: &'a [&'a FromDrop],
  ) -> Result<PreloaderResult<<<<<FromDrop as ModelBackedDrop>::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType, Value>, DropError>{
    info!(
      "{}.{}: eager-loading {} {}",
      pretty_type_name::<FromDrop>(),
      self.meta.field_name,
      drops.len(),
      pretty_type_name::<ToDrop>()
    );

    let preloader = self.association_type_adapter.preloader(context.clone());
    let preloader_result = preloader.preload(context.db(), drops).await?;
    let preloaded_drops = self
      .value_adapter
      .preloader_result_to_drops(&preloader_result);
    self.eager_load(context.clone(), &preloaded_drops).await?;
    Ok(preloader_result)
  }

  pub async fn eager_load(
    &self,
    context: C,
    preloaded_drops: &Vec<&ToDrop>,
  ) -> Result<(), DropError> {
    let association_preloads = self
      .meta
      .eager_loads
      .iter()
      .filter_map(|eager_load: &EagerLoadInstruction| {
        ASSOCIATION_REGISTRY
          .get_association::<ToDrop, Value, ToDrop::Context>(&eager_load.association_name)
      })
      .map(|assoc: &BoxAssociation<ToDrop, Value, ToDrop::Context>| {
        let ctx = context.clone();
        let preloaded_drops_slice = preloaded_drops.as_slice();
        assoc.preload(ctx, preloaded_drops_slice)
      });

    try_join_all(association_preloads).await.map(|_| ())
  }
}

impl<
    C: Context,
    FromDrop: ModelBackedDrop<Context = C> + LiquidDropWithID + Send + Sync + Clone,
    ToDrop: ModelBackedDrop<Context = C> + LiquidDropWithID + Send + Sync + 'static,
    Preloader: crate::preloaders::Preloader<FromDrop, ToDrop, DropPrimaryKeyValue<FromDrop>, Value> + Sync,
    Value: ValueView + Clone + Send + Sync,
  > ModelBackedDropAssociation for Association<FromDrop, ToDrop, Preloader, Value>
where
  FromDrop: Send + Sync + LiquidDropWithID + Into<DropResult<FromDrop>>,
  ToDrop: Send + Sync,
  DropPrimaryKeyValue<FromDrop>: Sync + Send + Eq + Clone + Hash + From<i64> + Copy,
  DropPrimaryKeyValue<ToDrop>: Eq + Hash,
  ToDrop::ID: From<i64>,
  i64: From<ToDrop::ID>,
{
  type FromDrop = FromDrop;
  type ToDrop = ToDrop;
  type Value = Value;
  type Preloader = Preloader;
  type PrimaryKeyValue = DropPrimaryKeyValue<ToDrop>;

  fn loader_result_to_drops(
    &self,
    result: Option<
      &dyn AssociationLoaderResult<
        <Self::FromDrop as ModelBackedDrop>::Model,
        <Self::ToDrop as ModelBackedDrop>::Model,
      >,
    >,
    from_drop: &Self::FromDrop,
  ) -> Result<Vec<Self::ToDrop>, DropError> {
    self
      .association_type_adapter
      .loader_result_to_drops(result, from_drop)
  }

  fn association_type_adapter(
    &self,
  ) -> &dyn ModelBackedDropAssociationAssociationTypeAdapter<
    FromDrop = Self::FromDrop,
    ToDrop = Self::ToDrop,
    Preloader = Self::Preloader,
    Value = Self::Value,
  > {
    self.association_type_adapter.as_ref()
  }

  fn value_adapter(
    &self,
  ) -> &dyn ModelBackedDropAssociationValueAdapter<
    Id = DropPrimaryKeyValue<Self::FromDrop>,
    ToDrop = Self::ToDrop,
    Value = Self::Value,
  > {
    self.value_adapter.as_ref()
  }

  fn get_once_cell(
    &self,
    cache: &<Self::FromDrop as LiquidDrop>::Cache,
  ) -> &OnceBox<DropResult<Self::Value>> {
    self.association_type_adapter.get_once_cell(cache)
  }

  fn preload<'a, 'life0, 'async_trait>(
    &'life0 self,
    context: <Self::FromDrop as ContextContainer>::Context,
    drops: &'a [&'a Self::FromDrop],
  ) -> core::pin::Pin<
    Box<
      dyn core::future::Future<
          Output = Result<PreloaderResult<Self::PrimaryKeyValue, Self::Value>, DropError>,
        > + core::marker::Send
        + 'async_trait,
    >,
  >
  where
    'a: 'async_trait,
    'life0: 'async_trait,
    Self: 'async_trait,
  {
    todo!()
  }

  fn get_meta(&self) -> &AssociationMeta<Self::FromDrop, Self::ToDrop> {
    &self.meta
  }
}
