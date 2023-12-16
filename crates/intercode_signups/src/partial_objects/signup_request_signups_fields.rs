use async_graphql::*;
use intercode_entities::{runs, signup_requests, user_con_profiles};
use intercode_graphql_core::{
  enums::SignupRequestState, load_one_by_model_id, loader_result_to_optional_single,
  model_backed_type, scalars::DateScalar,
};
use seawater::loaders::ExpectModel;

use super::SignupSignupsFields;

model_backed_type!(SignupRequestSignupsFields, signup_requests::Model);

impl SignupRequestSignupsFields {
  pub async fn replace_signup(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<SignupSignupsFields>, Error> {
    let loader_result = load_one_by_model_id!(signup_request_replace_signup, ctx, self)?;
    Ok(loader_result_to_optional_single!(
      loader_result,
      SignupSignupsFields
    ))
  }

  pub async fn result_signup(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<SignupSignupsFields>, Error> {
    let loader_result = load_one_by_model_id!(signup_request_result_signup, ctx, self)?;
    Ok(loader_result_to_optional_single!(
      loader_result,
      SignupSignupsFields
    ))
  }

  pub async fn target_run(&self, ctx: &Context<'_>) -> Result<runs::Model, Error> {
    let loader_result = load_one_by_model_id!(signup_request_target_run, ctx, self)?;
    Ok(loader_result.expect_one()?.clone())
  }

  pub async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<user_con_profiles::Model> {
    let loader_result = load_one_by_model_id!(signup_request_user_con_profile, ctx, self)?;
    Ok(loader_result.expect_one()?.clone())
  }
}

#[Object]
impl SignupRequestSignupsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "created_at")]
  async fn created_at(&self) -> Result<DateScalar> {
    self.model.created_at.try_into()
  }

  #[graphql(name = "requested_bucket_key")]
  async fn requested_bucket_key(&self) -> Option<&str> {
    self.model.requested_bucket_key.as_deref()
  }

  async fn state(&self) -> Result<SignupRequestState> {
    SignupRequestState::try_from(self.model.state.as_str()).map_err(Error::from)
  }
}
