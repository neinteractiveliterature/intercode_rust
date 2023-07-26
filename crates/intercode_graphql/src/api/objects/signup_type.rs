use async_graphql::*;
use intercode_entities::signups;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_policies::{
  policies::{SignupAction, SignupPolicy},
  ModelBackedTypeGuardablePolicy,
};
use intercode_signups::partial_objects::SignupSignupsFields;

use crate::{api::merged_objects::RunType, merged_model_backed_type};

use super::UserConProfileType;

model_backed_type!(SignupGlueFields, signups::Model);

#[Object(guard = "SignupPolicy::model_guard(SignupAction::Read, self)")]
impl SignupGlueFields {
  async fn run(&self, ctx: &Context<'_>) -> Result<RunType, Error> {
    SignupSignupsFields::from_type(self.clone())
      .run(ctx)
      .await
      .map(RunType::new)
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType, Error> {
    SignupSignupsFields::from_type(self.clone())
      .user_con_profile(ctx)
      .await
      .map(UserConProfileType::new)
  }
}

merged_model_backed_type!(
  SignupType,
  signups::Model,
  "Signup",
  SignupSignupsFields,
  SignupGlueFields
);
