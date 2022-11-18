use intercode_policies::{AuthorizationInfo, Policy};
use sea_orm::ModelTrait;

use crate::policy_guard::PolicyGuard;

pub trait ModelBackedType {
  type Model: ModelTrait;

  fn new(model: Self::Model) -> Self;
  fn get_model(&self) -> &Self::Model;

  fn policy_guard<P: Policy<AuthorizationInfo, Self::Model>>(
    &self,
    action: P::Action,
  ) -> PolicyGuard<P, Self::Model>
  where
    Self::Model: std::marker::Sync,
  {
    PolicyGuard::new(action, self.get_model())
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
