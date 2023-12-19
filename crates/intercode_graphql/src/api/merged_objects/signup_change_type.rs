use async_graphql::*;
use intercode_entities::signup_changes;
use intercode_graphql_core::model_backed_type;
use intercode_signups::partial_objects::{
  SignupChangeSignupsExtensions, SignupChangeSignupsFields,
};

use crate::{api::merged_objects::RunType, merged_model_backed_type};

use super::{SignupType, UserConProfileType};

model_backed_type!(SignupChangeGlueFields, signup_changes::Model);

impl SignupChangeSignupsExtensions for SignupChangeGlueFields {}

#[Object]
impl SignupChangeGlueFields {
  #[graphql(name = "previous_signup_change")]
  async fn previous_signup_change(&self, ctx: &Context<'_>) -> Result<Option<SignupChangeType>> {
    SignupChangeSignupsExtensions::previous_signup_change(self, ctx).await
  }

  async fn run(&self, ctx: &Context<'_>) -> Result<RunType> {
    SignupChangeSignupsExtensions::run(self, ctx).await
  }

  async fn signup(&self, ctx: &Context<'_>) -> Result<SignupType> {
    SignupChangeSignupsExtensions::signup(self, ctx).await
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    SignupChangeSignupsExtensions::user_con_profile(self, ctx).await
  }
}

merged_model_backed_type!(
  SignupChangeType,
  signup_changes::Model,
  "SignupChange",
  SignupChangeSignupsFields,
  SignupChangeGlueFields
);
