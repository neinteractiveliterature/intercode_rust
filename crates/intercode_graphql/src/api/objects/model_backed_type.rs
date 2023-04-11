use intercode_policies::{AuthorizationInfo, Policy};
use sea_orm::ModelTrait;

use crate::policy_guard::PolicyGuard;

pub trait ModelBackedType {
  type Model: ModelTrait;

  fn new(model: Self::Model) -> Self;
  fn get_model(&self) -> &Self::Model;

  fn simple_policy_guard<P: Policy<AuthorizationInfo, Self::Model>>(
    &self,
    action: P::Action,
  ) -> PolicyGuard<P, Self::Model, Self::Model>
  where
    Self::Model: std::marker::Sync,
  {
    PolicyGuard::new(action, self.get_model(), move |model, _ctx| {
      let model = model.clone();
      Box::pin(async { Ok(model) })
    })
  }
}

#[macro_export]
macro_rules! model_backed_type {
  ($type_name: ident, $model_type: ty) => {
    #[derive(Clone, Debug)]
    pub struct $type_name {
      model: $model_type,
    }

    impl $crate::api::objects::ModelBackedType for $type_name {
      type Model = $model_type;

      fn new(model: $model_type) -> Self {
        $type_name { model }
      }

      fn get_model(&self) -> &$model_type {
        &self.model
      }
    }
  };
}

#[macro_export]
macro_rules! load_one_by_id {
  ($loader: ident, $ctx: ident, $id: expr) => {
    $ctx
      .data::<$crate::QueryData>()?
      .loaders()
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
macro_rules! loader_result_to_optional_single {
  ($loader_result: ident, $type: ty) => {
    $loader_result.try_one().cloned().map(<$type>::new)
  };
}

#[macro_export]
macro_rules! loader_result_to_required_single {
  ($loader_result: ident, $type: ty) => {
    <$type>::new($loader_result.expect_one()?.clone())
  };
}

#[macro_export]
macro_rules! loader_result_to_many {
  ($loader_result: ident, $type: ty) => {
    $loader_result
      .expect_models()?
      .iter()
      .cloned()
      .map(<$type>::new)
      .collect()
  };
}
