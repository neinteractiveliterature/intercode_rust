use async_graphql::*;
use intercode_entities::signup_requests;

use crate::{
  api::scalars::DateScalar, load_one_by_model_id, loader_result_to_optional_single,
  loader_result_to_required_single, model_backed_type,
};

use super::{RunType, SignupType, UserConProfileType};

model_backed_type!(SignupRequestType, signup_requests::Model);

#[Object(name = "SignupRequest")]
impl SignupRequestType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "created_at")]
  async fn created_at(&self) -> DateScalar {
    self.model.created_at.into()
  }

  #[graphql(name = "replace_signup")]
  async fn replace_signup(&self, ctx: &Context<'_>) -> Result<Option<SignupType>, Error> {
    let loader_result = load_one_by_model_id!(signup_request_replace_signup, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, SignupType))
  }

  #[graphql(name = "result_signup")]
  async fn result_signup(&self, ctx: &Context<'_>) -> Result<Option<SignupType>, Error> {
    let loader_result = load_one_by_model_id!(signup_request_result_signup, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, SignupType))
  }

  #[graphql(name = "requested_bucket_key")]
  async fn requested_bucket_key(&self) -> Option<&str> {
    self.model.requested_bucket_key.as_deref()
  }

  async fn state(&self) -> &str {
    &self.model.state
  }

  #[graphql(name = "target_run")]
  async fn target_run(&self, ctx: &Context<'_>) -> Result<RunType, Error> {
    let loader_result = load_one_by_model_id!(signup_request_target_run, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, RunType))
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    let loader_result = load_one_by_model_id!(signup_request_user_con_profile, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      UserConProfileType
    ))
  }
}
