use async_graphql::*;
use intercode_conventions::{
  partial_objects::UserActivityAlertConventionsFields, policies::UserActivityAlertPolicy,
};
use intercode_entities::user_activity_alerts;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};

use crate::{
  api::merged_objects::{NotificationDestinationType, UserType},
  merged_model_backed_type,
};

model_backed_type!(UserActivityAlertGlueFields, user_activity_alerts::Model);

#[Object(guard = "UserActivityAlertPolicy::model_guard(ReadManageAction::Read, self)")]
impl UserActivityAlertGlueFields {
  #[graphql(name = "notification_destinations")]
  async fn notification_destinations(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<NotificationDestinationType>> {
    UserActivityAlertConventionsFields::from_type(self.clone())
      .notification_destinations(ctx)
      .await
      .map(|res| {
        res
          .into_iter()
          .map(NotificationDestinationType::new)
          .collect()
      })
  }

  async fn user(&self, ctx: &Context<'_>) -> Result<Option<UserType>> {
    UserActivityAlertConventionsFields::from_type(self.clone())
      .user(ctx)
      .await
      .map(|res| res.map(UserType::new))
  }
}

merged_model_backed_type!(
  UserActivityAlertType,
  user_activity_alerts::Model,
  "UserActivityAlert",
  UserActivityAlertConventionsFields,
  UserActivityAlertGlueFields
);
