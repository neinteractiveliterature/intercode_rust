use std::{future::Future, pin::Pin};

use async_graphql::{Context, ErrorExtensions, Guard, Result};
use async_trait::async_trait;
use intercode_policies::{AuthorizationInfo, Policy};

use crate::query_data::QueryData;

pub trait GetResourceFn<'a, M: Send + Sync + 'static, R: Send + Sync>:
  for<'b> Fn(&'a M, &'b Context<'_>) -> Pin<Box<dyn Future<Output = Result<R>> + Send + Sync + 'b>>
  + Send
  + Sync
  + 'a
{
}

impl<'a, M: Send + Sync + 'static, R: Send + Sync, F> GetResourceFn<'a, M, R> for F where
  F: for<'b> Fn(
      &'a M,
      &'b Context<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<R>> + Send + Sync + 'b>>
    + Send
    + Sync
    + 'a
{
}

pub struct PolicyGuard<
  'a,
  P: Policy<AuthorizationInfo, R>,
  R: Send + Sync,
  M: Send + Sync + 'static,
> {
  action: P::Action,
  model: &'a M,
  get_resource: Pin<Box<dyn GetResourceFn<'a, M, R>>>,
}

impl<'a, P: Policy<AuthorizationInfo, R>, R: Send + Sync, M: Send + Sync + 'static>
  PolicyGuard<'a, P, R, M>
{
  pub fn new<F: GetResourceFn<'a, M, R>>(action: P::Action, model: &'a M, get_resource: F) -> Self {
    Self {
      action,
      model,
      get_resource: Box::pin(get_resource),
    }
  }
}

#[async_trait]
impl<'a, P: Policy<AuthorizationInfo, R>, R: Send + Sync, M: Send + Sync + 'static> Guard
  for PolicyGuard<'a, P, R, M>
{
  async fn check(&self, ctx: &Context<'_>) -> Result<()> {
    let principal = ctx.data::<AuthorizationInfo>()?;
    let resource = (self.get_resource)(self.model, ctx).await?;
    let permitted = P::action_permitted(principal, &self.action, &resource).await?;

    match permitted {
      true => Ok(()),
      false => {
        let error = async_graphql::Error::new("Permission denied").extend_with(|_err, ext| {
          ext.set(
            "code",
            if ctx
              .data::<QueryData>()
              .ok()
              .and_then(|qd| qd.current_user())
              .is_none()
            {
              "NOT_AUTHENTICATED"
            } else {
              "NOT_AUTHORIZED"
            },
          )
        });

        Err(error)
      }
    }
  }
}
