use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{oauth_applications, staff_positions, user_con_profiles, users};
use intercode_graphql_core::{lax_id::LaxId, query_data::QueryData};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{
  model_action_permitted::model_action_permitted,
  policies::{ConventionAction, ConventionPolicy, UserConProfileAction, UserConProfilePolicy},
  AuthorizationInfo, Policy, ReadManageAction,
};
use seawater::loaders::ExpectModel;

use crate::policies::{OAuthApplicationPolicy, StaffPositionPolicy, UserAction, UserPolicy};

pub struct AbilityUsersFields {
  authorization_info: Arc<AuthorizationInfo>,
}

impl AbilityUsersFields {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }

  pub fn into_authorization_info(self) -> Arc<AuthorizationInfo> {
    self.authorization_info
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
impl AbilityUsersFields {
  #[graphql(name = "can_manage_oauth_applications")]
  async fn can_manage_oauth_applications(&self) -> Result<bool> {
    let authorization_info = self.authorization_info.as_ref();
    Ok(
      OAuthApplicationPolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Manage,
        &oauth_applications::Model::default(),
      )
      .await?,
    )
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

    let Some(convention) = convention else {
      return Ok(false);
    };

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

  #[graphql(name = "can_read_users")]
  async fn can_read_users(&self) -> Result<bool, Error> {
    Ok(
      UserPolicy::action_permitted(
        &self.authorization_info,
        &UserAction::Read,
        &users::Model::default(),
      )
      .await?,
    )
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
