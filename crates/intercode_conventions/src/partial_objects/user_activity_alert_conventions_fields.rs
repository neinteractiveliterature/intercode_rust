use crate::policies::UserActivityAlertPolicy;
use async_graphql::*;
use intercode_entities::{notification_destinations, user_activity_alerts, users};
use intercode_graphql_core::{load_one_by_model_id, model_backed_type};
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};
use seawater::loaders::{ExpectModel, ExpectModels};

model_backed_type!(
  UserActivityAlertConventionsFields,
  user_activity_alerts::Model
);

impl UserActivityAlertConventionsFields {
  pub async fn notification_destinations(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<notification_destinations::Model>> {
    let loader_result =
      load_one_by_model_id!(user_activity_alert_notification_destinations, ctx, self)?;
    loader_result.expect_models().cloned()
  }

  pub async fn user(&self, ctx: &Context<'_>) -> Result<Option<users::Model>> {
    let loader_result = load_one_by_model_id!(user_activity_alert_user, ctx, self)?;
    Ok(loader_result.try_one().cloned())
  }
}

#[Object(guard = "UserActivityAlertPolicy::model_guard(ReadManageAction::Read, self)")]
impl UserActivityAlertConventionsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn email(&self) -> Option<&str> {
    self.model.email.as_deref()
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
}
