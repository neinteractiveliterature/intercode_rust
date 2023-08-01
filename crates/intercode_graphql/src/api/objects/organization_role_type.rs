use async_graphql::*;
use intercode_entities::organization_roles;
use intercode_graphql_core::{load_one_by_model_id, loader_result_to_many, model_backed_type};

use crate::api::merged_objects::PermissionType;

use super::UserType;
model_backed_type!(OrganizationRoleType, organization_roles::Model);

#[Object(name = "OrganizationRole")]
impl OrganizationRoleType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }

  async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<PermissionType>> {
    let loader_result = load_one_by_model_id!(organization_role_permissions, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, PermissionType))
  }

  async fn users(&self, ctx: &Context<'_>) -> Result<Vec<UserType>> {
    let loader_result = load_one_by_model_id!(organization_role_users, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, UserType))
  }
}
