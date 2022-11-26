use std::sync::Arc;

use async_graphql::{Context, Guard, Result};
use async_session::async_trait;
use intercode_policies::{AuthorizationInfo, Policy};

pub struct PolicyGuard<'a, P: Policy<AuthorizationInfo, R>, R: Send + Sync> {
  action: P::Action,
  resource: &'a R,
}

impl<'a, P: Policy<AuthorizationInfo, R>, R: Send + Sync> PolicyGuard<'a, P, R> {
  pub fn new(action: P::Action, resource: &'a R) -> Self {
    Self { action, resource }
  }
}

#[async_trait]
impl<'a, P: Policy<AuthorizationInfo, R>, R: Send + Sync> Guard for PolicyGuard<'a, P, R> {
  async fn check(&self, ctx: &Context<'_>) -> Result<()> {
    let principal = ctx.data::<Arc<AuthorizationInfo>>()?;
    P::action_permitted(principal, &self.action, self.resource).await?;
    Ok(())
  }
}
