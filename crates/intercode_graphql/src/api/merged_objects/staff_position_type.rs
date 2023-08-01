use async_graphql::*;
use intercode_entities::staff_positions;
use intercode_graphql_core::{load_one_by_model_id, loader_result_to_many, model_backed_type};
use intercode_users::partial_objects::StaffPositionUsersFields;

use crate::{
  api::merged_objects::{PermissionType, UserConProfileType},
  merged_model_backed_type,
};

model_backed_type!(StaffPositionGlueFields, staff_positions::Model);

#[Object]
impl StaffPositionGlueFields {
  async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<PermissionType>> {
    let loader_result = load_one_by_model_id!(staff_position_permissions, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, PermissionType))
  }

  #[graphql(name = "user_con_profiles")]
  async fn user_con_profiles(&self, ctx: &Context<'_>) -> Result<Vec<UserConProfileType>> {
    let loader_result = load_one_by_model_id!(staff_position_user_con_profiles, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, UserConProfileType))
  }
}

merged_model_backed_type!(
  StaffPositionType,
  staff_positions::Model,
  "StaffPosition",
  StaffPositionUsersFields,
  StaffPositionGlueFields
);
