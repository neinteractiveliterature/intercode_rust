use crate::LiquidDrop;
use sea_orm::{EntityTrait, ModelTrait, PrimaryKeyTrait};

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
}

#[macro_export]
macro_rules! model_backed_drop {
  ($type_name: ident, $model_type: ty, $context_type: ty) => {
    #[derive(Debug, Clone)]
    pub struct $type_name {
      model: $model_type,
      #[allow(dead_code)]
      context: $context_type,
    }

    impl $crate::ModelBackedDrop for $type_name {
      type Model = $model_type;

      fn new(model: $model_type, context: $context_type) -> Self {
        $type_name { model, context }
      }

      fn get_model(&self) -> &$model_type {
        &self.model
      }
    }
  };
}
