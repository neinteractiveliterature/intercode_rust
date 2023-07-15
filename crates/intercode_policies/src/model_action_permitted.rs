use crate::{AuthorizationInfo, Policy};
use async_graphql::{Context, Error};
use std::borrow::Borrow;

pub async fn model_action_permitted<
  'a,
  P: Policy<AuthorizationInfo, M>,
  M: Send + Sync + 'a,
  R: Borrow<M>,
>(
  authorization_info: &AuthorizationInfo,
  _policy: P,
  ctx: &'a Context<'_>,
  action: &P::Action,
  get_model: impl FnOnce(&'a Context<'_>) -> Result<Option<R>, Error>,
) -> Result<bool, Error> {
  let model_ref = get_model(ctx)?;

  if let Some(model_ref) = model_ref {
    Ok(P::action_permitted(authorization_info, action, model_ref.borrow()).await?)
  } else {
    Ok(false)
  }
}
