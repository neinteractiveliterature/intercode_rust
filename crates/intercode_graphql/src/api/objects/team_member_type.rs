use crate::QueryData;
use async_graphql::*;
use intercode_entities::team_members;
use seawater::loaders::ExpectModels;

use crate::model_backed_type;

use super::{EventType, ModelBackedType, UserConProfileType};
model_backed_type!(TeamMemberType, team_members::Model);

#[Object(name = "TeamMember")]
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
    if query_data.current_user.is_none()
      || !self.model.show_email.unwrap_or(false)
      || !self.model.display.unwrap_or(false)
    {
      return Ok(None);
    }

    let user_con_profile_result = query_data
      .loaders
      .team_member_user_con_profile
      .load_one(self.model.id)
      .await?;

    let user_result = query_data
      .loaders
      .user_con_profile_user
      .load_one(user_con_profile_result.expect_one()?.id)
      .await?;

    Ok(Some(user_result.expect_one()?.email.clone()))
  }

  async fn event(&self, ctx: &Context<'_>) -> Result<EventType, Error> {
    let loader = &ctx.data::<QueryData>()?.loaders.team_member_event;

    let result = loader.load_one(self.model.id).await?;
    let event = result.expect_one()?;

    Ok(EventType::new(event.to_owned()))
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType, Error> {
    let loader = &ctx
      .data::<QueryData>()?
      .loaders
      .team_member_user_con_profile;

    let result = loader.load_one(self.model.id).await?;
    let user_con_profile = result.expect_one()?;

    Ok(UserConProfileType::new(user_con_profile.to_owned()))
  }
}
