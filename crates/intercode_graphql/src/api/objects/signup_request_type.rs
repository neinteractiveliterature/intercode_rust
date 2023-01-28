use async_graphql::*;
use intercode_entities::signup_requests;
use seawater::loaders::ExpectModels;

use crate::{model_backed_type, QueryData};

use super::{ModelBackedType, RunType, SignupType};

model_backed_type!(SignupRequestType, signup_requests::Model);

#[Object(name = "SignupRequest")]
impl SignupRequestType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "replace_signup")]
  async fn replace_signup(&self, ctx: &Context<'_>) -> Result<Option<SignupType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(
      query_data
        .loaders
        .signup_request_replace_signup()
        .load_one(self.model.id)
        .await?
        .try_one()
        .cloned()
        .map(SignupType::new),
    )
  }

  #[graphql(name = "result_signup")]
  async fn result_signup(&self, ctx: &Context<'_>) -> Result<Option<SignupType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(
      query_data
        .loaders
        .signup_request_result_signup()
        .load_one(self.model.id)
        .await?
        .try_one()
        .cloned()
        .map(SignupType::new),
    )
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
    let query_data = ctx.data::<QueryData>()?;
    Ok(RunType::new(
      query_data
        .loaders
        .signup_request_target_run()
        .load_one(self.model.id)
        .await?
        .expect_one()?
        .clone(),
    ))
  }
}
