use async_graphql::*;
use intercode_entities::{organization_roles, organizations, permissions, users};
use intercode_graphql_core::{load_one_by_model_id, model_backed_type};
use seawater::loaders::{ExpectModel, ExpectModels};

model_backed_type!(OrganizationRoleConventionsFields, organization_roles::Model);

impl OrganizationRoleConventionsFields {
  pub async fn organization(&self, ctx: &Context<'_>) -> Result<organizations::Model> {
    let loader_result = load_one_by_model_id!(organization_role_organization, ctx, self)?;
    loader_result.expect_one().cloned()
  }

  pub async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<permissions::Model>> {
    let loader_result = load_one_by_model_id!(organization_role_permissions, ctx, self)?;
    loader_result.expect_models().cloned()
  }

  pub async fn users(&self, ctx: &Context<'_>) -> Result<Vec<users::Model>> {
    let loader_result = load_one_by_model_id!(organization_role_users, ctx, self)?;
    loader_result.expect_models().cloned()
  }
}

#[Object]
impl OrganizationRoleConventionsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn name(&self) -> &str {
    self.model.name.as_deref().unwrap_or_default()
  }
}
