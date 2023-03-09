use crate::{DropResult, LiquidDrop};
use liquid::ValueView;
use sea_orm::{EntityTrait, Linked, ModelTrait, PrimaryKeyTrait, Related};

use crate::preloaders::{EntityLinkPreloaderBuilder, EntityRelationPreloaderBuilder};

pub type DropModel<D> = <D as ModelBackedDrop>::Model;
pub type DropEntity<D> = <DropModel<D> as ModelTrait>::Entity;
pub type DropPrimaryKey<D> = <DropEntity<D> as EntityTrait>::PrimaryKey;
pub type DropPrimaryKeyValue<D> = <DropPrimaryKey<D> as PrimaryKeyTrait>::ValueType;

pub trait ModelBackedDrop
where
  Self: LiquidDrop,
{
  type Model: ModelTrait;

  fn new(model: Self::Model, context: Self::Context) -> Self;
  fn get_model(&self) -> &Self::Model;

  fn link_preloader<ToDrop: ModelBackedDrop, Value: ValueView>(
    link: impl Linked<FromEntity = DropEntity<Self>, ToEntity = DropEntity<ToDrop>>
      + Send
      + Sync
      + 'static,
    pk_column: DropPrimaryKey<Self>,
  ) -> EntityLinkPreloaderBuilder<Self, ToDrop, Value, Self::Context>
  where
    Self: Send + Sync + Clone,
    ToDrop: Send + Sync + Clone,
    DropPrimaryKeyValue<Self>: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
    Value: Into<DropResult<Value>> + Clone,
    DropPrimaryKeyValue<Self>: Clone,
  {
    EntityLinkPreloaderBuilder::new(link, pk_column)
  }

  fn relation_preloader<ToDrop: ModelBackedDrop, Value: ValueView>(
    pk_column: DropPrimaryKey<Self>,
  ) -> EntityRelationPreloaderBuilder<Self, ToDrop, Value, Self::Context>
  where
    Self: Send + Sync + Clone,
    ToDrop: Send + Sync + Clone,
    DropEntity<Self>: Related<DropEntity<ToDrop>>,
    DropPrimaryKeyValue<Self>: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
    Value: Into<DropResult<Value>> + Clone,
    DropPrimaryKeyValue<Self>: Clone,
  {
    EntityRelationPreloaderBuilder::new(pk_column)
  }
}

#[macro_export]
macro_rules! model_backed_drop {
  ($type_name: ident, $model_type: ty, $context_type: ty) => {
    #[::seawater::liquid_drop_struct]
    pub struct $type_name {
      model: $model_type,
      #[allow(dead_code)]
      context: $context_type,
    }

    impl $crate::ModelBackedDrop for $type_name {
      type Model = $model_type;

      fn new(model: $model_type, context: $context_type) -> Self {
        $type_name {
          model,
          context,
          drop_cache: ::std::default::Default::default(),
        }
      }

      fn get_model(&self) -> &$model_type {
        &self.model
      }
    }
  };
}
