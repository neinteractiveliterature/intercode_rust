use async_graphql::{Context, Object, Result};
use intercode_entities::users;
use intercode_graphql_core::model_backed_type;
use intercode_users::partial_objects::{UserUsersExtensions, UserUsersFields};

use crate::merged_model_backed_type;

use super::{EventProposalType, UserConProfileType};

model_backed_type!(UserGlueFields, users::Model);

impl UserUsersExtensions for UserGlueFields {}

#[Object]
impl UserGlueFields {
  #[graphql(name = "event_proposals")]
  async fn event_proposals(&self, ctx: &Context<'_>) -> Result<Vec<EventProposalType>> {
    UserUsersExtensions::event_proposals(self, ctx).await
  }

  #[graphql(name = "user_con_profiles")]
  async fn user_con_profiles(&self, ctx: &Context<'_>) -> Result<Vec<UserConProfileType>> {
    UserUsersExtensions::user_con_profiles(self, ctx).await
  }
}

merged_model_backed_type!(
  UserType,
  users::Model,
  "User",
  UserUsersFields,
  UserGlueFields
);
