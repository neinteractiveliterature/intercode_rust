use async_graphql::*;
use intercode_entities::permissions;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_graphql_loaders::{
  permissioned_models_loader::PermissionedModel, permissioned_roles_loader::PermissionedRole,
};
use intercode_users::partial_objects::PermissionUsersFields;

use crate::{
  api::{
    merged_objects::EventCategoryType,
    objects::{CmsContentGroupType, ConventionType, OrganizationRoleType, StaffPositionType},
    unions::{PermissionedModelType, PermissionedRoleType},
  },
  merged_model_backed_type,
};

model_backed_type!(PermissionGlueFields, permissions::Model);

#[Object]
impl PermissionGlueFields {
  async fn model(&self, ctx: &Context<'_>) -> Result<PermissionedModelType> {
    let model = PermissionUsersFields::from_type(self.clone())
      .model(ctx)
      .await?;

    Ok(match model {
      PermissionedModel::CmsContentGroup(model) => {
        PermissionedModelType::CmsContentGroup(CmsContentGroupType::new(model))
      }
      PermissionedModel::Convention(model) => {
        PermissionedModelType::Convention(ConventionType::new(model))
      }
      PermissionedModel::EventCategory(model) => {
        PermissionedModelType::EventCategory(EventCategoryType::new(model))
      }
    })
  }

  async fn role(&self, ctx: &Context<'_>) -> Result<PermissionedRoleType> {
    let role = PermissionUsersFields::from_type(self.clone())
      .role(ctx)
      .await?;

    Ok(match role {
      PermissionedRole::OrganizationRole(role) => {
        PermissionedRoleType::OrganizationRole(OrganizationRoleType::new(role))
      }
      PermissionedRole::StaffPosition(role) => {
        PermissionedRoleType::StaffPosition(StaffPositionType::new(role))
      }
    })
  }
}

merged_model_backed_type!(
  PermissionType,
  permissions::Model,
  "Permission",
  PermissionUsersFields,
  PermissionGlueFields
);
