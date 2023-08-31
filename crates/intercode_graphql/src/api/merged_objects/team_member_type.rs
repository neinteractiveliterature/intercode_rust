use async_graphql::*;
use intercode_entities::team_members;
use intercode_events::partial_objects::TeamMemberEventsFields;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_policies::{
  policies::TeamMemberPolicy, ModelBackedTypeGuardablePolicy, ReadManageAction,
};

use crate::{api::merged_objects::EventType, merged_model_backed_type};

use super::UserConProfileType;

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

merged_model_backed_type!(
  TeamMemberType,
  team_members::Model,
  "TeamMember",
  TeamMemberGlueFields,
  TeamMemberEventsFields
);
