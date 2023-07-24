use async_graphql::*;
use intercode_entities::team_members;
use intercode_events::partial_objects::TeamMemberEventsFields;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_policies::{
  policies::TeamMemberPolicy, ModelBackedTypeGuardablePolicy, ReadManageAction,
};

use crate::api::{merged_objects::EventType, objects::UserConProfileType};

model_backed_type!(TeamMemberGlueFields, team_members::Model);

#[Object(guard = "TeamMemberPolicy::model_guard(ReadManageAction::Read, self)")]
impl TeamMemberGlueFields {
  async fn event(&self, ctx: &Context<'_>) -> Result<EventType, Error> {
    TeamMemberEventsFields::from_type(self.clone())
      .event(ctx)
      .await
      .map(EventType::from_type)
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType, Error> {
    TeamMemberEventsFields::from_type(self.clone())
      .user_con_profile(ctx)
      .await
      .map(UserConProfileType::new)
  }
}

#[derive(MergedObject)]
#[graphql(name = "TeamMember")]
pub struct TeamMemberType(TeamMemberGlueFields, TeamMemberEventsFields);

impl ModelBackedType for TeamMemberType {
  type Model = team_members::Model;

  fn new(model: Self::Model) -> Self {
    Self(
      TeamMemberGlueFields::new(model.clone()),
      TeamMemberEventsFields::new(model),
    )
  }

  fn get_model(&self) -> &Self::Model {
    self.0.get_model()
  }

  fn into_model(self) -> Self::Model {
    self.0.into_model()
  }
}
