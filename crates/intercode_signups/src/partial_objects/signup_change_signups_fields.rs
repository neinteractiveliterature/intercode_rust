use std::str::FromStr;

use async_graphql::{Context, Object, Result, ID};
use axum::async_trait;
use intercode_entities::{
  links::SignupChangeToPreviousSignupChange, runs, signup_changes, signups, user_con_profiles,
};
use intercode_graphql_core::{
  enums::{SignupChangeAction, SignupState},
  load_one_by_model_id, loader_result_to_required_single, model_backed_type,
  query_data::QueryData,
  scalars::DateScalar,
  ModelBackedType,
};
use sea_orm::ModelTrait;

#[async_trait]
pub trait SignupChangeSignupsExtensions
where
  Self: ModelBackedType<Model = signup_changes::Model>,
{
  async fn previous_signup_change<T: ModelBackedType<Model = signup_changes::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<T>> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      self
        .get_model()
        .find_linked(SignupChangeToPreviousSignupChange)
        .one(query_data.db())
        .await?
        .map(T::new),
    )
  }

  async fn run<T: ModelBackedType<Model = runs::Model>>(&self, ctx: &Context<'_>) -> Result<T> {
    let loader_result = load_one_by_model_id!(signup_change_run, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }

  async fn signup<T: ModelBackedType<Model = signups::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<T> {
    let loader_result = load_one_by_model_id!(signup_change_signup, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }

  async fn user_con_profile<T: ModelBackedType<Model = user_con_profiles::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<T> {
    let loader_result = load_one_by_model_id!(signup_change_user_con_profile, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }
}

model_backed_type!(SignupChangeSignupsFields, signup_changes::Model);

#[Object]
impl SignupChangeSignupsFields {
  async fn id(&self) -> ID {
    ID(self.model.id.to_string())
  }

  async fn action(&self) -> Result<SignupChangeAction> {
    Ok(SignupChangeAction::from_str(&self.model.action)?)
  }

  #[graphql(name = "bucket_key")]
  async fn bucket_key(&self) -> Option<&str> {
    self.model.bucket_key.as_deref()
  }

  async fn counted(&self) -> bool {
    self.model.counted.unwrap_or(false)
  }

  #[graphql(name = "created_at")]
  async fn created_at(&self) -> Result<DateScalar> {
    self.model.created_at.try_into()
  }

  async fn state(&self) -> Result<SignupState> {
    Ok(SignupState::from_str(&self.model.state)?)
  }

  #[graphql(name = "updated_at")]
  async fn updated_at(&self) -> Result<DateScalar> {
    self.model.updated_at.try_into()
  }
}
