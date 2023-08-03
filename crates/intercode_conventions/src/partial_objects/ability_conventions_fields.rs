use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{conventions, departments, organizations, user_activity_alerts};
use intercode_graphql_core::query_data::QueryData;
use intercode_policies::{
  model_action_permitted::model_action_permitted,
  policies::{ConventionAction, ConventionPolicy},
  AuthorizationInfo, Policy, ReadManageAction,
};

use crate::policies::{DepartmentPolicy, OrganizationPolicy, UserActivityAlertPolicy};

pub struct AbilityConventionsFields {
  authorization_info: Arc<AuthorizationInfo>,
}

impl AbilityConventionsFields {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }
}

#[Object]
impl AbilityConventionsFields {
  #[graphql(name = "can_manage_conventions")]
  async fn can_manage_conventions(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ConventionPolicy,
      ctx,
      &ConventionAction::Update,
      |_ctx| Ok(Some(conventions::Model::default())),
    )
    .await
  }

  #[graphql(name = "can_update_convention")]
  async fn can_update_convention(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ConventionPolicy,
      ctx,
      &ConventionAction::Update,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_update_departments")]
  async fn can_update_departments(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = self.authorization_info.as_ref();
    let Some(convention) = ctx.data::<QueryData>()?.convention() else {
      return Ok(false);
    };

    Ok(
      DepartmentPolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Manage,
        &departments::Model {
          convention_id: convention.id,
          ..Default::default()
        },
      )
      .await?,
    )
  }

  #[graphql(name = "can_read_organizations")]
  async fn can_read_organizations(&self) -> Result<bool> {
    let authorization_info = self.authorization_info.as_ref();

    Ok(
      OrganizationPolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Read,
        &organizations::Model::default(),
      )
      .await?,
    )
  }

  #[graphql(name = "can_read_user_activity_alerts")]
  async fn can_read_user_activity_alerts(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = self.authorization_info.as_ref();
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention)= convention else {
      return Ok(false);
    };

    Ok(
      UserActivityAlertPolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Read,
        &user_activity_alerts::Model {
          convention_id: convention.id,
          ..Default::default()
        },
      )
      .await?,
    )
  }
}
