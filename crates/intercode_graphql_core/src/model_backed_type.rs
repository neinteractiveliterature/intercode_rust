use std::sync::Arc;

use sea_orm::ModelTrait;

pub trait ModelBackedType {
  type Model: ModelTrait;

  fn new(model: Self::Model) -> Self;
  fn get_model(&self) -> &Self::Model;
  fn from_arc(arc: Arc<Self::Model>) -> Self;
  fn clone_model_arc(&self) -> Arc<Self::Model>;

  fn into_type<Other: ModelBackedType<Model = Self::Model>>(self) -> Other
  where
    Self: Sized,
  {
    Other::from_arc(self.clone_model_arc())
  }

  fn from_type<Other: ModelBackedType<Model = Self::Model>>(other: Other) -> Self
  where
    Self: Sized,
  {
    other.into_type()
  }
}

#[macro_export]
macro_rules! model_backed_type {
  ($type_name: ident, $model_type: ty) => {
    #[derive(Clone, Debug)]
    pub struct $type_name {
      model: ::std::sync::Arc<$model_type>,
    }

    impl $crate::ModelBackedType for $type_name {
      type Model = $model_type;

      fn new(model: $model_type) -> Self {
        Self {
          model: ::std::sync::Arc::new(model),
        }
      }

      fn from_arc(arc: ::std::sync::Arc<$model_type>) -> Self {
        Self { model: arc }
      }

      fn get_model(&self) -> &$model_type {
        self.model.as_ref()
      }

      fn clone_model_arc(&self) -> ::std::sync::Arc<$model_type> {
        self.model.clone()
      }
    }
  };
}

#[macro_export]
macro_rules! load_one_by_id {
  ($loader: ident, $ctx: ident, $id: expr) => {
    $ctx
      .data::<::std::sync::Arc<::intercode_graphql_loaders::LoaderManager>>()?
      .$loader()
      .load_one($id)
      .await
  };
}

#[macro_export]
macro_rules! load_one_by_model_id {
  ($loader: ident, $ctx: ident, $self: expr) => {
    $crate::load_one_by_id!($loader, $ctx, $self.model.id)
  };
}

#[macro_export]
macro_rules! load_many_by_ids {
  ($loader: ident, $ctx: ident, $ids: expr) => {
    $ctx
      .data::<::std::sync::Arc<::intercode_graphql_loaders::LoaderManager>>()?
      .$loader()
      .load_many($ids)
      .await
  };
}

#[macro_export]
macro_rules! load_many_by_model_ids {
  ($loader: ident, $ctx: ident, $models: expr) => {
    $crate::load_many_by_ids!($loader, $ctx, $models.map(|model| model.id))
  };
}

#[macro_export]
macro_rules! loader_result_to_optional_single {
  ($loader_result: ident, $type: ty) => {
    ::seawater::loaders::ExpectModel::try_one(&$loader_result)
      .cloned()
      .map(<$type as $crate::ModelBackedType>::new)
  };
}

#[macro_export]
macro_rules! loader_result_to_required_single {
  ($loader_result: ident, $type: ty) => {
    <$type as $crate::ModelBackedType>::new(
      ::seawater::loaders::ExpectModel::expect_one(&$loader_result)?.clone(),
    )
  };
}

#[macro_export]
macro_rules! loader_result_to_many {
  ($loader_result: ident, $type: ty) => {
    ::seawater::loaders::ExpectModels::expect_models(&$loader_result)?
      .iter()
      .cloned()
      .map(<$type as $crate::ModelBackedType>::new)
      .collect()
  };
}

#[macro_export]
macro_rules! loader_result_map_to_required_map {
  ($loader_result_map: expr) => {
    $loader_result_map
      .into_iter()
      .map(|(id, model_result)| model_result.expect_one().map(|model| (id, model.clone())))
      .collect::<Result<HashMap<_, _>, _>>()
  };
}
