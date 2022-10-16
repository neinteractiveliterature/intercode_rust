use intercode_graphql::SchemaData;
use lazy_liquid_value_view::{DropResult, LiquidDrop};
use liquid::ValueView;
use sea_orm::{EntityTrait, Linked, ModelTrait, PrimaryKeyTrait, Related};

use crate::preloaders::{EntityLinkPreloaderBuilder, EntityRelationPreloaderBuilder};

pub trait ModelBackedDrop
where
  Self: LiquidDrop,
{
  type Model: ModelTrait;

  fn new(model: Self::Model, schema_data: SchemaData) -> Self;
  fn get_model(&self) -> &Self::Model;

  fn link_preloader<ToDrop: ModelBackedDrop, Value: ValueView>(
    link: impl Linked<FromEntity = <Self::Model as ModelTrait>::Entity, ToEntity = <ToDrop::Model as ModelTrait>::Entity> + Send + Sync + 'static,
    pk_column: <<Self::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey
  ) -> EntityLinkPreloaderBuilder::<Self, ToDrop, Value>
  where
    Self: Send + Sync,
    ToDrop: Send + Sync,
    <<<Self::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
    Value: Into<DropResult<Value>>,
    <<<Self::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone
  {
    EntityLinkPreloaderBuilder::new(link, pk_column)
  }

  fn relation_preloader<ToDrop: ModelBackedDrop, Value: ValueView>(
    pk_column: <<Self::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey
  ) -> EntityRelationPreloaderBuilder::<Self, ToDrop, Value>
  where
    Self: Send + Sync,
    ToDrop: Send + Sync,
    <Self::Model as ModelTrait>::Entity: Related<<ToDrop::Model as ModelTrait>::Entity>,
    <<<Self::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
    Value: Into<DropResult<Value>>,
    <<<Self::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone
  {
    EntityRelationPreloaderBuilder::new(pk_column)
  }
}

#[macro_export]
macro_rules! model_backed_drop {
  ($type_name: ident, $model_type: ty) => {
    #[liquid_drop_struct]
    pub struct $type_name {
      model: $model_type,
      #[allow(dead_code)]
      schema_data: ::intercode_graphql::SchemaData,
    }

    impl $crate::ModelBackedDrop for $type_name {
      type Model = $model_type;

      fn new(model: $model_type, schema_data: ::intercode_graphql::SchemaData) -> Self {
        $type_name {
          model,
          schema_data,
          drop_cache: ::std::default::Default::default(),
        }
      }

      fn get_model(&self) -> &$model_type {
        &self.model
      }
    }
  };
}
