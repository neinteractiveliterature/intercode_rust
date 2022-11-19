use async_graphql::*;
use intercode_entities::{conventions, rooms};
use intercode_policies::{
  policies::{ConventionAction, ConventionPolicy, RoomPolicy},
  AuthorizationInfo, Policy, ReadManageAction,
};

use crate::QueryData;

pub struct AbilityType;

// TODO just about everything here
#[Object(name = "Ability")]
impl AbilityType {
  #[graphql(name = "can_manage_conventions")]
  async fn can_manage_conventions(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;

    Ok(
      ConventionPolicy::action_permitted(
        authorization_info,
        &ConventionAction::Update,
        &conventions::Model::default(),
      )
      .await?,
    )
  }

  #[graphql(name = "can_read_schedule")]
  async fn can_read_schedule(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_schedule_with_counts")]
  async fn can_read_schedule_with_counts(&self) -> bool {
    false
  }
  #[graphql(name = "can_list_events")]
  async fn can_list_events(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_user_con_profiles")]
  async fn can_read_user_con_profiles(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let query_data = ctx.data::<QueryData>()?;
    let convention = query_data.convention.as_ref().as_ref();

    match convention {
      Some(convention) => Ok(
        ConventionPolicy::action_permitted(
          authorization_info,
          &ConventionAction::ViewAttendees,
          convention,
        )
        .await?,
      ),
      None => Ok(false),
    }
  }
  #[graphql(name = "can_update_convention")]
  async fn can_update_convention(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let query_data = ctx.data::<QueryData>()?;
    let convention = query_data.convention.as_ref().as_ref();

    match convention {
      Some(convention) => Ok(
        ConventionPolicy::action_permitted(
          authorization_info,
          &ConventionAction::Update,
          convention,
        )
        .await?,
      ),
      None => Ok(false),
    }
  }
  #[graphql(name = "can_update_departments")]
  async fn can_update_departments(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_email_routes")]
  async fn can_manage_email_routes(&self) -> bool {
    false
  }
  #[graphql(name = "can_update_event_categories")]
  async fn can_update_event_categories(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_event_proposals")]
  async fn can_read_event_proposals(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_runs")]
  async fn can_manage_runs(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_forms")]
  async fn can_manage_forms(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_any_mailing_list")]
  async fn can_read_any_mailing_list(&self) -> bool {
    false
  }
  #[graphql(name = "can_update_notification_templates")]
  async fn can_update_notification_templates(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_oauth_applications")]
  async fn can_manage_oauth_applications(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_reports")]
  async fn can_read_reports(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let query_data = ctx.data::<QueryData>()?;
    let convention = query_data.convention.as_ref().as_ref();

    match convention {
      Some(convention) => Ok(
        ConventionPolicy::action_permitted(
          authorization_info,
          &ConventionAction::ViewReports,
          convention,
        )
        .await?,
      ),
      None => Ok(false),
    }
  }
  #[graphql(name = "can_manage_rooms")]
  async fn can_manage_rooms(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let query_data = ctx.data::<QueryData>()?;

    RoomPolicy::action_permitted(
      authorization_info,
      &ReadManageAction::Manage,
      &rooms::Model {
        convention_id: query_data.convention.as_ref().as_ref().map(|con| con.id),
        ..Default::default()
      },
    )
    .await
    .map_err(|e| e.into())
  }
  #[graphql(name = "can_manage_signups")]
  async fn can_manage_signups(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_any_cms_content")]
  async fn can_manage_any_cms_content(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_staff_positions")]
  async fn can_manage_staff_positions(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_orders")]
  async fn can_read_orders(&self) -> bool {
    false
  }
  #[graphql(name = "can_manage_ticket_types")]
  async fn can_manage_ticket_types(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_user_activity_alerts")]
  async fn can_read_user_activity_alerts(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_organizations")]
  async fn can_read_organizations(&self) -> bool {
    false
  }
  #[graphql(name = "can_read_users")]
  async fn can_read_users(&self) -> bool {
    false
  }
}
