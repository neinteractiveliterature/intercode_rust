use async_graphql::{async_trait::async_trait, futures_util::try_join, Error};
use intercode_entities::{conventions, notification_templates};
use intercode_graphql_core::liquid_renderer::LiquidRenderer;
use sea_orm::{ColumnTrait, DbErr, ModelTrait, QueryFilter};
use seawater::ConnectionWrapper;

use crate::{
  NotificationCategoryConfig, NotificationDestination, NotificationEventConfig,
  RenderedEmailNotification, RenderedNotification, RenderedNotificationContent,
  NOTIFICATIONS_CONFIG,
};

#[async_trait]
pub trait Notifier: Send + Sync {
  fn get_convention(&self) -> &conventions::Model;
  fn get_category_key(&self) -> &str;
  fn get_event_key(&self) -> &str;
  fn get_liquid_assigns(&self) -> liquid::Object;
  async fn get_destinations(
    &self,
    db: &ConnectionWrapper,
  ) -> Result<Vec<NotificationDestination>, Error>;

  fn get_category(&self) -> &NotificationCategoryConfig {
    NOTIFICATIONS_CONFIG
      .categories
      .get(self.get_category_key())
      .unwrap()
  }

  fn get_event(&self) -> &NotificationEventConfig {
    self
      .get_category()
      .events
      .get(self.get_event_key())
      .unwrap()
  }

  fn get_qualified_event_key(&self) -> String {
    format!("{}/{}", self.get_category().key, self.get_event().key)
  }

  async fn load_notification_template(
    &self,
    db: &ConnectionWrapper,
  ) -> Result<notification_templates::Model, DbErr> {
    let convention = self.get_convention();
    let qualified_event_key = self.get_qualified_event_key();
    convention
      .find_related(notification_templates::Entity)
      .filter(notification_templates::Column::EventKey.eq(&qualified_event_key))
      .one(db)
      .await
      .and_then(|model| {
        model.ok_or_else(|| {
          DbErr::RecordNotFound(format!(
            "Notification template for {} not found in {}",
            &qualified_event_key,
            convention.name.as_deref().unwrap_or_default()
          ))
        })
      })
  }

  async fn render_content(
    &self,
    content: &str,
    liquid_renderer: &dyn LiquidRenderer,
  ) -> Result<String, Error> {
    liquid_renderer
      .render_liquid(content, self.get_liquid_assigns(), None)
      .await
  }

  async fn render_email(
    &self,
    notification_template: &notification_templates::Model,
    liquid_renderer: &dyn LiquidRenderer,
    db: &ConnectionWrapper,
  ) -> Result<RenderedNotification, Error> {
    let (subject, body_html, body_text, destinations) = try_join!(
      self.render_content(
        notification_template.subject.as_deref().unwrap_or_default(),
        liquid_renderer,
      ),
      self.render_content(
        notification_template
          .body_html
          .as_deref()
          .unwrap_or_default(),
        liquid_renderer,
      ),
      self.render_content(
        notification_template
          .body_text
          .as_deref()
          .unwrap_or_default(),
        liquid_renderer,
      ),
      self.get_destinations(db)
    )?;

    let content = RenderedEmailNotification {
      subject,
      body_html,
      body_text,
    };

    Ok(RenderedNotification {
      content: RenderedNotificationContent::Email(content),
      destinations,
    })
  }

  async fn render_sms(
    &self,
    notification_template: &notification_templates::Model,
    liquid_renderer: &dyn LiquidRenderer,
    db: &ConnectionWrapper,
  ) -> Result<RenderedNotification, Error> {
    let (body_sms, destinations) = try_join!(
      self.render_content(
        notification_template
          .body_sms
          .as_deref()
          .unwrap_or_default(),
        liquid_renderer,
      ),
      self.get_destinations(db),
    )?;

    Ok(RenderedNotification {
      content: RenderedNotificationContent::Sms(body_sms),
      destinations,
    })
  }
}
