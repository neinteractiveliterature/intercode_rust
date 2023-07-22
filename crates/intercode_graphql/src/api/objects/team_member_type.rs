use std::sync::Arc;

use async_graphql::*;
use intercode_entities::team_members;
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_required_single, model_backed_type, query_data::QueryData,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{
  policies::TeamMemberPolicy, ModelBackedTypeGuardablePolicy, ReadManageAction,
};
use seawater::loaders::ExpectModel;

use super::{EventType, UserConProfileType};
model_backed_type!(TeamMemberType, team_members::Model);

#[Object(
  name = "TeamMember",
  guard = "TeamMemberPolicy::model_guard(ReadManageAction::Read, self)"
)]
impl TeamMemberType {
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

  async fn event(&self, ctx: &Context<'_>) -> Result<EventType, Error> {
    let loader_result = load_one_by_model_id!(team_member_event, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, EventType))
  }

  #[graphql(name = "receive_con_email")]
  async fn receive_con_email(&self) -> bool {
    self.model.receive_con_email.unwrap_or(false)
  }

  #[graphql(name = "receive_signup_email")]
  async fn receive_signup_email(&self) -> &str {
    &self.model.receive_signup_email
  }

  #[graphql(name = "show_email")]
  async fn show_email(&self) -> bool {
    self.model.show_email.unwrap_or(false)
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType, Error> {
    let loader_result = load_one_by_model_id!(team_member_user_con_profile, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      UserConProfileType
    ))
  }
}
