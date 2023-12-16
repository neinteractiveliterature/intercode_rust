use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{team_members, user_con_profiles};
use intercode_graphql_core::{
  enums::ReceiveSignupEmail, load_one_by_model_id, loader_result_to_required_single,
  model_backed_type, query_data::QueryData,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{
  policies::TeamMemberPolicy, ModelBackedTypeGuardablePolicy, ReadManageAction,
};
use seawater::loaders::ExpectModel;

use super::EventEventsFields;
model_backed_type!(TeamMemberEventsFields, team_members::Model);

impl TeamMemberEventsFields {
  pub async fn event(&self, ctx: &Context<'_>) -> Result<EventEventsFields, Error> {
    let loader_result = load_one_by_model_id!(team_member_event, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      EventEventsFields
    ))
  }

  pub async fn user_con_profile(
    &self,
    ctx: &Context<'_>,
  ) -> Result<user_con_profiles::Model, Error> {
    let loader_result = load_one_by_model_id!(team_member_user_con_profile, ctx, self)?;
    Ok(loader_result.expect_one()?.clone())
  }
}

#[Object(guard = "TeamMemberPolicy::model_guard(ReadManageAction::Read, self)")]
impl TeamMemberEventsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "display_team_member")]
  async fn display_team_member(&self) -> bool {
    self.model.display.unwrap_or(false)
  }

  async fn email(&self, ctx: &Context<'_>) -> Result<Option<String>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    if query_data.current_user().is_none()
      || !self.model.show_email.unwrap_or(false)
      || !self.model.display.unwrap_or(false)
    {
      return Ok(None);
    }

    let loaders = ctx.data::<Arc<LoaderManager>>()?;

    let user_con_profile_result = loaders
      .team_member_user_con_profile()
      .load_one(self.model.id)
      .await?;

    let user_result = loaders
      .user_con_profile_user()
      .load_one(user_con_profile_result.expect_one()?.id)
      .await?;

    Ok(Some(user_result.expect_one()?.email.clone()))
  }

  #[graphql(name = "receive_con_email")]
  async fn receive_con_email(&self) -> bool {
    self.model.receive_con_email.unwrap_or(false)
  }

  #[graphql(name = "receive_signup_email")]
  async fn receive_signup_email(&self) -> Result<ReceiveSignupEmail> {
    ReceiveSignupEmail::try_from(self.model.receive_signup_email.as_str()).map_err(Error::from)
  }

  #[graphql(name = "show_email")]
  async fn show_email(&self) -> bool {
    self.model.show_email.unwrap_or(false)
  }
}
