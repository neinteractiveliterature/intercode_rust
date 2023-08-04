use async_graphql::*;
use intercode_entities::staff_positions;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_users::partial_objects::StaffPositionUsersFields;

use crate::{
  api::merged_objects::{PermissionType, UserConProfileType},
  merged_model_backed_type,
};

model_backed_type!(StaffPositionGlueFields, staff_positions::Model);

#[Object]
impl StaffPositionGlueFields {
  async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<PermissionType>> {
    StaffPositionUsersFields::from_type(self.clone())
      .permissions(ctx)
      .await
      .map(|res| res.into_iter().map(PermissionType::new).collect())
  }

  #[graphql(name = "user_con_profiles")]
  async fn user_con_profiles(&self, ctx: &Context<'_>) -> Result<Vec<UserConProfileType>> {
    StaffPositionUsersFields::from_type(self.clone())
      .user_con_profiles(ctx)
      .await
      .map(|res| res.into_iter().map(UserConProfileType::from_type).collect())
  }
}

merged_model_backed_type!(
  StaffPositionType,
  staff_positions::Model,
  "StaffPosition",
  StaffPositionUsersFields,
  StaffPositionGlueFields
);
