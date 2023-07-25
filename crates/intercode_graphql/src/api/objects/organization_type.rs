use async_graphql::*;
use intercode_entities::{organization_roles, organizations};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, model_backed_type, ModelBackedType,
};
use intercode_policies::{
  policies::{OrganizationPolicy, OrganizationRolePolicy},
  AuthorizationInfo, ModelBackedTypeGuardablePolicy, Policy, ReadManageAction,
};

use super::{ConventionType, OrganizationRoleType};
model_backed_type!(OrganizationType, organizations::Model);

#[Object(
  name = "Organization",
  guard = "OrganizationPolicy::model_guard(ReadManageAction::Read, self)"
)]
impl OrganizationType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn conventions(&self, ctx: &Context<'_>) -> Result<Vec<ConventionType>> {
    let loader_result = load_one_by_model_id!(organization_conventions, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, ConventionType))
  }

  #[graphql(name = "current_ability_can_manage_access")]
  async fn current_ability_can_manage_access(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    Ok(
      OrganizationRolePolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Manage,
        &(
          self.get_model().clone(),
          organization_roles::Model {
            organization_id: Some(self.model.id),
            ..Default::default()
          },
        ),
      )
      .await?,
    )
  }

  async fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }

  #[graphql(name = "organization_roles")]
  async fn organization_roles(&self, ctx: &Context<'_>) -> Result<Vec<OrganizationRoleType>> {
    let loader_result = load_one_by_model_id!(organization_organization_roles, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, OrganizationRoleType))
  }
}
