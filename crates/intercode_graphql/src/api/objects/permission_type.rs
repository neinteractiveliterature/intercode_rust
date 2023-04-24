use async_graphql::*;
use intercode_entities::{
  model_ext::permissions::{PermissionedModelRef, PermissionedRoleRef},
  permissions,
};
use seawater::loaders::ExpectModel;

use crate::{
  api::unions::{PermissionedModelType, PermissionedRoleType},
  loaders::permissioned_models_loader::PermissionedModel,
  loaders::permissioned_roles_loader::PermissionedRole,
  model_backed_type, QueryData,
};

use super::{
  CmsContentGroupType, ConventionType, EventCategoryType, ModelBackedType, OrganizationRoleType,
  StaffPositionType,
};
model_backed_type!(PermissionType, permissions::Model);

#[Object(name = "Permission")]
impl PermissionType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn model(&self, ctx: &Context<'_>) -> Result<PermissionedModelType> {
    let model_ref: PermissionedModelRef = (&self.model).try_into()?;
    let result = ctx
      .data::<QueryData>()?
      .loaders()
      .permissioned_models
      .load_one(model_ref)
      .await?;

    result.expect_model().map(|model| match model {
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

  async fn permission(&self) -> &str {
    &self.model.permission
  }

  async fn role(&self, ctx: &Context<'_>) -> Result<PermissionedRoleType> {
    let role_ref: PermissionedRoleRef = (&self.model).try_into()?;
    let result = ctx
      .data::<QueryData>()?
      .loaders()
      .permissioned_roles
      .load_one(role_ref)
      .await?;

    result.expect_model().map(|role| match role {
      PermissionedRole::OrganizationRole(role) => {
        PermissionedRoleType::OrganizationRole(OrganizationRoleType::new(role))
      }
      PermissionedRole::StaffPosition(role) => {
        PermissionedRoleType::StaffPosition(StaffPositionType::new(role))
      }
    })
  }
}
