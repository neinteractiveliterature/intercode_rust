use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{
  model_ext::permissions::{PermissionedModelRef, PermissionedRoleRef},
  permissions,
};
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_graphql_loaders::{
  permissioned_models_loader::PermissionedModel, permissioned_roles_loader::PermissionedRole,
  LoaderManager,
};
use seawater::loaders::ExpectModel;

model_backed_type!(PermissionUsersFields, permissions::Model);

impl PermissionUsersFields {
  pub async fn model(&self, ctx: &Context<'_>) -> Result<PermissionedModel> {
    let model_ref: PermissionedModelRef = self.get_model().try_into()?;
    let result = ctx
      .data::<Arc<LoaderManager>>()?
      .permissioned_models
      .load_one(model_ref)
      .await?;

    Ok(result.expect_one()?.clone())
  }

  pub async fn role(&self, ctx: &Context<'_>) -> Result<PermissionedRole> {
    let role_ref: PermissionedRoleRef = self.get_model().try_into()?;
    let result = ctx
      .data::<Arc<LoaderManager>>()?
      .permissioned_roles
      .load_one(role_ref)
      .await?;

    Ok(result.expect_one()?.clone())
  }
}

#[Object]
impl PermissionUsersFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn permission(&self) -> &str {
    &self.model.permission
  }
}
