use std::sync::Arc;

use async_graphql::*;
use intercode_cms::api::partial_objects::AbilityCmsFields;
use intercode_entities::{
  conventions, departments, email_routes, organizations, staff_positions, user_activity_alerts,
  user_con_profiles,
};
use intercode_events::partial_objects::AbilityEventsFields;
use intercode_forms::partial_objects::AbilityFormsFields;
use intercode_graphql_core::{lax_id::LaxId, query_data::QueryData};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{
  model_action_permitted::model_action_permitted,
  policies::{
    ConventionAction, ConventionPolicy, DepartmentPolicy, EmailRoutePolicy, OrganizationPolicy,
    StaffPositionPolicy, UserActivityAlertPolicy, UserConProfileAction, UserConProfilePolicy,
  },
  AuthorizationInfo, Policy, ReadManageAction,
};
use intercode_reporting::partial_objects::AbilityReportingFields;
use intercode_signups::partial_objects::AbilitySignupsFields;
use intercode_store::partial_objects::AbilityStoreFields;
use seawater::loaders::ExpectModel;

pub struct AbilityApiFields {
  authorization_info: Arc<AuthorizationInfo>,
}

impl AbilityApiFields {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }

  async fn can_perform_user_con_profile_action(
    &self,
    ctx: &Context<'_>,
    user_con_profile_id: ID,
    action: &UserConProfileAction,
  ) -> Result<bool> {
    let loader_result = ctx
      .data::<Arc<LoaderManager>>()?
      .user_con_profiles_by_id()
      .load_one(LaxId::parse(user_con_profile_id)?)
      .await?;

    let user_con_profile = loader_result.expect_one()?;

    model_action_permitted(
      self.authorization_info.as_ref(),
      UserConProfilePolicy,
      ctx,
      action,
      |_ctx| Ok(Some(user_con_profile)),
    )
    .await
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

  #[graphql(name = "can_read_user_con_profiles")]
  async fn can_read_user_con_profiles(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ConventionPolicy,
      ctx,
      &ConventionAction::ViewAttendees,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_create_user_con_profiles")]
  async fn can_create_user_con_profiles(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let convention = ctx.data::<QueryData>()?.convention();

    let Some(convention) = convention else { return Ok(false); };

    let user_con_profile = user_con_profiles::Model {
      convention_id: convention.id,
      ..Default::default()
    };

    model_action_permitted(
      self.authorization_info.as_ref(),
      UserConProfilePolicy,
      ctx,
      &UserConProfileAction::Create,
      |_ctx| Ok(Some(user_con_profile)),
    )
    .await
  }

  #[graphql(name = "can_become_user_con_profile")]
  async fn can_become_user_con_profile(
    &self,
    ctx: &Context<'_>,
    user_con_profile_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_user_con_profile_action(ctx, user_con_profile_id, &UserConProfileAction::Become)
      .await
  }

  #[graphql(name = "can_delete_user_con_profile")]
  async fn can_delete_user_con_profile(
    &self,
    ctx: &Context<'_>,
    user_con_profile_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_user_con_profile_action(ctx, user_con_profile_id, &UserConProfileAction::Delete)
      .await
  }

  #[graphql(name = "can_update_user_con_profile")]
  async fn can_update_user_con_profile(
    &self,
    ctx: &Context<'_>,
    user_con_profile_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_user_con_profile_action(ctx, user_con_profile_id, &UserConProfileAction::Update)
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

  #[graphql(name = "can_read_users")]
  async fn can_read_users(&self) -> bool {
    // TODO
    false
  }

  #[graphql(name = "can_withdraw_all_user_con_profile_signups")]
  async fn can_withdraw_all_user_con_profile_signups(
    &self,
    ctx: &Context<'_>,
    user_con_profile_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_user_con_profile_action(
        ctx,
        user_con_profile_id,
        &UserConProfileAction::WithdrawAllSignups,
      )
      .await
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
      AbilityApiFields::new(authorization_info),
    )
  }
}
