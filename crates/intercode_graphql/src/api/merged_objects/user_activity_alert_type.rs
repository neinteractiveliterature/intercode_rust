use async_graphql::*;
use intercode_conventions::{
  partial_objects::{UserActivityAlertConventionsExtension, UserActivityAlertConventionsFields},
  policies::UserActivityAlertPolicy,
};
use intercode_entities::user_activity_alerts;
use intercode_graphql_core::model_backed_type;
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};

use crate::{
  api::merged_objects::{NotificationDestinationType, UserType},
  merged_model_backed_type,
};

use super::ConventionType;

model_backed_type!(UserActivityAlertGlueFields, user_activity_alerts::Model);

impl UserActivityAlertConventionsExtension for UserActivityAlertGlueFields {}

#[Object(guard = "UserActivityAlertPolicy::model_guard(ReadManageAction::Read, self)")]
impl UserActivityAlertGlueFields {
  async fn convention(&self, ctx: &Context<'_>) -> Result<ConventionType> {
    UserActivityAlertConventionsExtension::convention(self, ctx).await
  }

  #[graphql(name = "notification_destinations")]
  async fn notification_destinations(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<NotificationDestinationType>> {
    UserActivityAlertConventionsExtension::notification_destinations(self, ctx).await
  }

  async fn user(&self, ctx: &Context<'_>) -> Result<Option<UserType>> {
    UserActivityAlertConventionsExtension::user(self, ctx).await
  }
}

merged_model_backed_type!(
  UserActivityAlertType,
  user_activity_alerts::Model,
  "UserActivityAlert",
  UserActivityAlertConventionsFields,
  UserActivityAlertGlueFields
);
