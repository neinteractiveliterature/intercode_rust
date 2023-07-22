use crate::api::policies::NotificationTemplatePolicy;
use async_graphql::*;
use intercode_entities::notification_templates;
use intercode_graphql_core::model_backed_type;
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};

model_backed_type!(NotificationTemplateType, notification_templates::Model);

#[Object(
  name = "NotificationTemplate",
  guard = "NotificationTemplatePolicy::model_guard(ReadManageAction::Read, self)"
)]
impl NotificationTemplateType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "body_html")]
  async fn body_html(&self) -> Option<&str> {
    self.model.body_html.as_deref()
  }

  #[graphql(name = "body_sms")]
  async fn body_sms(&self) -> Option<&str> {
    self.model.body_sms.as_deref()
  }

  #[graphql(name = "body_text")]
  async fn body_text(&self) -> Option<&str> {
    self.model.body_text.as_deref()
  }

  #[graphql(name = "event_key")]
  async fn event_key(&self) -> &str {
    &self.model.event_key
  }

  async fn subject(&self) -> Option<&str> {
    self.model.subject.as_deref()
  }
}
