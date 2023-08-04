use async_graphql::*;
use intercode_entities::permissions;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_users::partial_objects::PermissionUsersFields;

use crate::{
  api::unions::{PermissionedModelType, PermissionedRoleType},
  merged_model_backed_type,
};

model_backed_type!(PermissionGlueFields, permissions::Model);

#[Object]
impl PermissionGlueFields {
  async fn model(&self, ctx: &Context<'_>) -> Result<PermissionedModelType> {
    PermissionUsersFields::from_type(self.clone())
      .model(ctx)
      .await
      .map(PermissionedModelType::from)
  }

  async fn role(&self, ctx: &Context<'_>) -> Result<PermissionedRoleType> {
    PermissionUsersFields::from_type(self.clone())
      .role(ctx)
      .await
      .map(PermissionedRoleType::from)
  }
}

merged_model_backed_type!(
  PermissionType,
  permissions::Model,
  "Permission",
  PermissionUsersFields,
  PermissionGlueFields
);
