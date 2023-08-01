use async_graphql::*;
use intercode_entities::user_activity_alerts;
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, loader_result_to_optional_single, model_backed_type,
};
use intercode_policies::{
  policies::UserActivityAlertPolicy, ModelBackedTypeGuardablePolicy, ReadManageAction,
};

use crate::api::merged_objects::UserType;

use super::notification_destination_type::NotificationDestinationType;
model_backed_type!(UserActivityAlertType, user_activity_alerts::Model);

#[Object(
  name = "UserActivityAlert",
  guard = "UserActivityAlertPolicy::model_guard(ReadManageAction::Read, self)"
)]
impl UserActivityAlertType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn email(&self) -> Option<&str> {
    self.model.email.as_deref()
  }

  #[graphql(name = "notification_destinations")]
  async fn notification_destinations(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<NotificationDestinationType>> {
    let loader_result =
      load_one_by_model_id!(user_activity_alert_notification_destinations, ctx, self)?;
    Ok(loader_result_to_many!(
      loader_result,
      NotificationDestinationType
    ))
  }

  #[graphql(name = "partial_name")]
  async fn partial_name(&self) -> Option<&str> {
    self.model.partial_name.as_deref()
  }

  #[graphql(name = "trigger_on_ticket_create")]
  async fn trigger_on_ticket_create(&self) -> bool {
    self.model.trigger_on_ticket_create
  }

  #[graphql(name = "trigger_on_user_con_profile_create")]
  async fn trigger_on_user_con_profile_create(&self) -> bool {
    self.model.trigger_on_user_con_profile_create
  }

  async fn user(&self, ctx: &Context<'_>) -> Result<Option<UserType>> {
    let loader_result = load_one_by_model_id!(user_activity_alert_user, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, UserType))
  }
}
