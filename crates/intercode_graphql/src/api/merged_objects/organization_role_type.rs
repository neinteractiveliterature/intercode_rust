use async_graphql::*;
use intercode_conventions::partial_objects::OrganizationRoleConventionsFields;
use intercode_entities::organization_roles;
use intercode_graphql_core::{model_backed_type, ModelBackedType};

use crate::{
  api::merged_objects::{PermissionType, UserType},
  merged_model_backed_type,
};

model_backed_type!(OrganizationRoleGlueFields, organization_roles::Model);

#[Object]
impl OrganizationRoleGlueFields {
  async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<PermissionType>> {
    OrganizationRoleConventionsFields::from_type(self.clone())
      .permissions(ctx)
      .await
      .map(|res| res.into_iter().map(PermissionType::new).collect())
  }

  async fn users(&self, ctx: &Context<'_>) -> Result<Vec<UserType>> {
    OrganizationRoleConventionsFields::from_type(self.clone())
      .users(ctx)
      .await
      .map(|res| res.into_iter().map(UserType::new).collect())
  }
}

merged_model_backed_type!(
  OrganizationRoleType,
  organization_roles::Model,
  "OrganizationRole",
  OrganizationRoleConventionsFields,
  OrganizationRoleGlueFields
);
