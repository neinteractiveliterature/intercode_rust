use std::{future::Future, pin::Pin};

use async_graphql::{Context, ErrorExtensions, Guard, Result};
use async_trait::async_trait;
use intercode_graphql_core::query_data::QueryData;

use crate::{AuthorizationInfo, Policy};

pub trait GetResourceFn<M: Send + Sync + 'static, R: Send + Sync>:
  for<'a, 'b> Fn(
    &'a M,
    &'b Context<'_>,
  ) -> Pin<Box<dyn Future<Output = Result<R>> + Send + Sync + 'b>>
  + Send
  + Sync
{
}

impl<M: Send + Sync + 'static, R: Send + Sync, F> GetResourceFn<M, R> for F where
  F: for<'a, 'b> Fn(
      &'a M,
      &'b Context<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<R>> + Send + Sync + 'b>>
    + Send
    + Sync
{
}

#[async_trait]
pub trait PolicyGuard<'a, P: Policy<AuthorizationInfo, R>, R: Send + Sync, M: Send + Sync + 'static>
{
  fn new(action: P::Action, model: &'a M) -> Self
  where
    Self: Sized;

  fn get_action(&self) -> &P::Action;
  fn get_model(&self) -> &M;
  async fn get_resource(&self, model: &M, ctx: &Context<'_>) -> Result<R>;

  async fn check(&self, ctx: &Context<'_>) -> Result<()> {
    let principal = ctx.data::<AuthorizationInfo>()?;
    let resource = self.get_resource(self.get_model(), ctx).await?;
    let permitted = P::action_permitted(principal, self.get_action(), &resource).await?;

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

#[async_trait]
impl<'a, P: Policy<AuthorizationInfo, R>, R: Send + Sync, M: Send + Sync + 'static> Guard
  for Box<dyn PolicyGuard<'a, P, R, M> + Send + Sync>
{
  async fn check(&self, ctx: &Context<'_>) -> Result<()> {
    self.as_ref().check(ctx).await
  }
}
