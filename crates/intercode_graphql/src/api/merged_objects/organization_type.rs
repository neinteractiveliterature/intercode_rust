use async_graphql::*;
use intercode_conventions::{
  partial_objects::OrganizationConventionsFields, policies::OrganizationPolicy,
};
use intercode_entities::organizations;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};

use crate::{
  api::merged_objects::{ConventionType, OrganizationRoleType},
  merged_model_backed_type,
};

model_backed_type!(OrganizationGlueFields, organizations::Model);

#[Object(guard = "OrganizationPolicy::model_guard(ReadManageAction::Read, self)")]
impl OrganizationGlueFields {
  async fn conventions(&self, ctx: &Context<'_>) -> Result<Vec<ConventionType>> {
    OrganizationConventionsFields::from_type(self.clone())
      .conventions(ctx)
      .await
      .map(|res| res.into_iter().map(ConventionType::from_type).collect())
  }

  #[graphql(name = "organization_roles")]
  async fn organization_roles(&self, ctx: &Context<'_>) -> Result<Vec<OrganizationRoleType>> {
    OrganizationConventionsFields::from_type(self.clone())
      .organization_roles(ctx)
      .await
      .map(|res| {
        res
          .into_iter()
          .map(OrganizationRoleType::from_type)
          .collect()
      })
  }
}

merged_model_backed_type!(
  OrganizationType,
  organizations::Model,
  "Organization",
  OrganizationConventionsFields,
  OrganizationGlueFields
);
