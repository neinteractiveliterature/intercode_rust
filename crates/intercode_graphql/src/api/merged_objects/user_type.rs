use async_graphql::{Context, Object, Result};
use intercode_entities::users;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_users::partial_objects::UserUsersFields;

use crate::merged_model_backed_type;

use super::UserConProfileType;

model_backed_type!(UserGlueFields, users::Model);

#[Object]
impl UserGlueFields {
  #[graphql(name = "user_con_profiles")]
  async fn user_con_profiles(&self, ctx: &Context<'_>) -> Result<Vec<UserConProfileType>> {
    UserConProfileType::from_many_future_result(
      UserUsersFields::from_type(self.clone()).user_con_profiles(ctx),
    )
    .await
  }
}

merged_model_backed_type!(
  UserType,
  users::Model,
  "User",
  UserUsersFields,
  UserGlueFields
);
