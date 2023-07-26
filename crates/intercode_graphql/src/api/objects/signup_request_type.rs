use async_graphql::*;
use intercode_entities::signup_requests;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_signups::partial_objects::SignupRequestSignupsFields;

use crate::{api::merged_objects::RunType, merged_model_backed_type};

use super::{SignupType, UserConProfileType};

model_backed_type!(SignupRequestGlueFields, signup_requests::Model);

#[Object]
impl SignupRequestGlueFields {
  #[graphql(name = "replace_signup")]
  async fn replace_signup(&self, ctx: &Context<'_>) -> Result<Option<SignupType>, Error> {
    SignupRequestSignupsFields::from_type(self.clone())
      .replace_signup(ctx)
      .await
      .map(|opt| opt.map(SignupType::from_type))
  }

  #[graphql(name = "result_signup")]
  async fn result_signup(&self, ctx: &Context<'_>) -> Result<Option<SignupType>, Error> {
    SignupRequestSignupsFields::from_type(self.clone())
      .result_signup(ctx)
      .await
      .map(|opt| opt.map(SignupType::from_type))
  }

  #[graphql(name = "target_run")]
  async fn target_run(&self, ctx: &Context<'_>) -> Result<RunType, Error> {
    SignupRequestSignupsFields::from_type(self.clone())
      .target_run(ctx)
      .await
      .map(RunType::new)
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    SignupRequestSignupsFields::from_type(self.clone())
      .user_con_profile(ctx)
      .await
      .map(UserConProfileType::new)
  }
}

merged_model_backed_type!(
  SignupRequestType,
  signup_requests::Model,
  "SignupRequest",
  SignupRequestSignupsFields,
  SignupRequestGlueFields
);
