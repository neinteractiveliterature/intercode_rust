use async_graphql::{Context, Error, Guard};
use async_trait::async_trait;
use sea_orm::ModelTrait;

use crate::{AuthorizationInfo, GuardablePolicy, Policy, PolicyGuard};

/// A PolicyGuard for policies where the model is the same as the resource
pub struct SimplePolicyGuard<M: sea_orm::ModelTrait + Sync, P: Policy<AuthorizationInfo, M>> {
  action: P::Action,
  model: M,
}

#[async_trait]
impl<'a, M: sea_orm::ModelTrait + Sync + 'static, P: Policy<AuthorizationInfo, M> + 'static>
  PolicyGuard<'a, P, M, M> for SimplePolicyGuard<M, P>
{
  fn new(action: P::Action, model: &'a M) -> Self {
    Self {
      action,
      model: model.clone(),
    }
  }

  fn get_action(&self) -> &P::Action {
    &self.action
  }

  fn get_model(&self) -> &M {
    &self.model
  }

  async fn get_resource(&self, model: &M, _ctx: &Context<'_>) -> Result<M, Error> {
    Ok(model.clone())
  }
}

#[async_trait]
impl<M: sea_orm::ModelTrait + Sync + 'static, P: Policy<AuthorizationInfo, M> + 'static> Guard
  for SimplePolicyGuard<M, P>
{
  async fn check(&self, ctx: &Context<'_>) -> Result<(), Error> {
    <Self as PolicyGuard<P, M, M>>::check(self, ctx).await
  }
}

pub trait SimpleGuardablePolicy<'a, Model: ModelTrait + Send + Sync + Clone + 'static>:
  GuardablePolicy<'a, Model, Model> + 'static
{
}

impl<'a, Model: ModelTrait + Send + Sync + Clone + 'static, P: SimpleGuardablePolicy<'a, Model>>
  GuardablePolicy<'a, Model, Model> for P
{
  type Guard = SimplePolicyGuard<Model, Self>;
}
