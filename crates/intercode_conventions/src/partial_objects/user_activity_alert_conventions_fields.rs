use crate::policies::UserActivityAlertPolicy;
use async_graphql::*;
use intercode_entities::{conventions, notification_destinations, user_activity_alerts, users};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, loader_result_to_optional_single,
  loader_result_to_required_single, model_backed_type, ModelBackedType,
};
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};
use sea_orm::prelude::async_trait::async_trait;

model_backed_type!(
  UserActivityAlertConventionsFields,
  user_activity_alerts::Model
);

#[async_trait]
pub trait UserActivityAlertConventionsExtension
where
  Self: ModelBackedType<Model = user_activity_alerts::Model>,
{
  async fn convention<T: ModelBackedType<Model = conventions::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<T> {
    let loader_result = load_one_by_model_id!(user_activity_alert_convention, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }

  async fn notification_destinations<
    T: ModelBackedType<Model = notification_destinations::Model>,
  >(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>> {
    let loader_result =
      load_one_by_model_id!(user_activity_alert_notification_destinations, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }

  async fn user<T: ModelBackedType<Model = users::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<T>> {
    let loader_result = load_one_by_model_id!(user_activity_alert_user, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, T))
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
