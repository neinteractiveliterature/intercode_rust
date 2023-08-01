use std::sync::Arc;

use async_graphql::*;
use intercode_cms::api::partial_objects::AbilityCmsFields;
use intercode_entities::{
  conventions, departments, email_routes, organizations, staff_positions, user_activity_alerts,
};
use intercode_events::partial_objects::AbilityEventsFields;
use intercode_forms::partial_objects::AbilityFormsFields;
use intercode_graphql_core::query_data::QueryData;

use intercode_policies::{
  model_action_permitted::model_action_permitted,
  policies::{
    ConventionAction, ConventionPolicy, DepartmentPolicy, EmailRoutePolicy, OrganizationPolicy,
    StaffPositionPolicy, UserActivityAlertPolicy,
  },
  AuthorizationInfo, Policy, ReadManageAction,
};
use intercode_reporting::partial_objects::AbilityReportingFields;
use intercode_signups::partial_objects::AbilitySignupsFields;
use intercode_store::partial_objects::AbilityStoreFields;
use intercode_users::partial_objects::AbilityUsersFields;

pub struct AbilityApiFields {
  authorization_info: Arc<AuthorizationInfo>,
}

impl AbilityApiFields {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }
}

#[Object]
impl AbilityApiFields {
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

  #[graphql(name = "can_manage_email_routes")]
  async fn can_manage_email_routes(&self) -> Result<bool> {
    Ok(
      EmailRoutePolicy::action_permitted(
        self.authorization_info.as_ref(),
        &ReadManageAction::Manage,
        &email_routes::Model::default(),
      )
      .await?,
    )
  }

  #[graphql(name = "can_manage_oauth_applications")]
  async fn can_manage_oauth_applications(&self) -> bool {
    // TODO
    false
  }

  #[graphql(name = "can_manage_staff_positions")]
  async fn can_manage_staff_positions(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = self.authorization_info.as_ref();
    let convention = ctx.data::<QueryData>()?.convention();
    Ok(
      StaffPositionPolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Manage,
        &staff_positions::Model {
          convention_id: convention.map(|c| c.id),
          ..Default::default()
        },
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
}

#[derive(MergedObject)]
#[graphql(name = "Ability")]
pub struct AbilityType(
  AbilityEventsFields,
  AbilityStoreFields,
  AbilityCmsFields,
  AbilityFormsFields,
  AbilityReportingFields,
  AbilitySignupsFields,
  AbilityUsersFields,
  AbilityApiFields,
);

impl AbilityType {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self(
      AbilityEventsFields::new(authorization_info.clone()),
      AbilityStoreFields::new(authorization_info.clone()),
      AbilityCmsFields::new(authorization_info.clone()),
      AbilityFormsFields::new(authorization_info.clone()),
      AbilityReportingFields::new(authorization_info.clone()),
      AbilitySignupsFields::new(authorization_info.clone()),
      AbilityUsersFields::new(authorization_info.clone()),
      AbilityApiFields::new(authorization_info),
    )
  }
}

impl From<AbilityUsersFields> for AbilityType {
  fn from(value: AbilityUsersFields) -> Self {
    Self::new(value.into_authorization_info())
  }
}
