use crate::{policy_guard::PolicyGuard, QueryData};
use async_graphql::*;
use intercode_entities::{conventions, events, team_members};
use intercode_policies::{policies::TeamMemberPolicy, ReadManageAction};
use seawater::loaders::ExpectModel;

use crate::model_backed_type;

use super::{EventType, ModelBackedType, UserConProfileType};
model_backed_type!(TeamMemberType, team_members::Model);

impl TeamMemberType {
  fn policy_guard(
    &self,
    action: ReadManageAction,
  ) -> PolicyGuard<
    '_,
    TeamMemberPolicy,
    (conventions::Model, events::Model, team_members::Model),
    team_members::Model,
  > {
    PolicyGuard::new(action, &self.model, move |model, ctx| {
      let model = model.clone();
      let ctx = ctx;
      let query_data = ctx.data::<QueryData>();

      Box::pin(async {
        let query_data = query_data?;
        let event_loader = query_data.loaders().team_member_event();
        let convention_loader = query_data.loaders().event_convention();
        let event_result = event_loader.load_one(model.id).await?;
        let event = event_result.expect_one()?;
        let convention_result = convention_loader.load_one(event.id).await?;
        let convention = convention_result.expect_one()?;

        Ok((convention.clone(), event.clone(), model))
      })
    })
  }
}

#[Object(
  name = "TeamMember",
  guard = "self.policy_guard(ReadManageAction::Read)"
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

    let user_con_profile_result = query_data
      .loaders()
      .team_member_user_con_profile()
      .load_one(self.model.id)
      .await?;

    let user_result = query_data
      .loaders()
      .user_con_profile_user()
      .load_one(user_con_profile_result.expect_one()?.id)
      .await?;

    Ok(Some(user_result.expect_one()?.email.clone()))
  }

  async fn event(&self, ctx: &Context<'_>) -> Result<EventType, Error> {
    let loader = ctx.data::<QueryData>()?.loaders().team_member_event();

    let result = loader.load_one(self.model.id).await?;
    let event = result.expect_one()?;

    Ok(EventType::new(event.to_owned()))
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
    let loader = &ctx
      .data::<QueryData>()?
      .loaders()
      .team_member_user_con_profile();

    let result = loader.load_one(self.model.id).await?;
    let user_con_profile = result.expect_one()?;

    Ok(UserConProfileType::new(user_con_profile.to_owned()))
  }
}
