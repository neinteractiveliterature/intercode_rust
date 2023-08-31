use std::env;

use async_graphql::{async_trait::async_trait, futures_util::try_join, Error};
use chrono::{Duration, Utc};
use intercode_entities::{conventions, notification_templates};
use intercode_graphql_core::liquid_renderer::LiquidRenderer;
use sea_orm::{ColumnTrait, DbErr, ModelTrait, QueryFilter};
use seawater::ConnectionWrapper;

use crate::{
  NotificationCategoryConfig, NotificationDestination, NotificationEventConfig,
  RenderedNotification, NOTIFICATIONS_CONFIG,
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

  async fn render(
    &self,
    notification_template: &notification_templates::Model,
    liquid_renderer: &dyn LiquidRenderer,
    db: &ConnectionWrapper,
  ) -> Result<RenderedNotification, Error> {
    let (subject, body_html, body_text, body_sms, destinations) = try_join!(
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
      self.render_content(
        notification_template
          .body_sms
          .as_deref()
          .unwrap_or_default(),
        liquid_renderer,
      ),
      self.get_destinations(db)
    )?;

    Ok(RenderedNotification {
      destinations,
      subject,
      body_html: if body_html.trim().is_empty() {
        None
      } else {
        Some(body_html)
      },
      body_text: if body_text.trim().is_empty() {
        None
      } else {
        Some(body_text)
      },
      body_sms: if body_sms.trim().is_empty() {
        None
      } else {
        Some(body_sms)
      },
    })
  }

  fn should_send_sms(&self) -> bool {
    let event = self.get_event();

    if event.sends_sms {
      if env::var("TWILIO_SMS_DEBUG_DESTINATION").is_ok() {
        return true;
      }

      if env::var("TWILIO_SMS_NUMBER").is_err() {
        return false;
      }

      let convention = self.get_convention();
      if let (Some(starts_at), Some(ends_at)) = (convention.starts_at, convention.ends_at) {
        let now = Utc::now();
        (starts_at.and_utc() - Duration::hours(24)) <= now && (ends_at.and_utc() > now)
      } else {
        false
      }
    } else {
      false
    }
  }

  async fn send(
    &self,
    notification_template: &notification_templates::Model,
    liquid_renderer: &dyn LiquidRenderer,
    db: &ConnectionWrapper,
  ) -> Result<(), Error> {
    let convention = self.get_convention();
    let rendered = self
      .render(notification_template, liquid_renderer, db)
      .await?;

    try_join!(rendered.send_email(&convention.email_from, db), async {
      if self.should_send_sms() {
        rendered
          .send_sms(&env::var("TWILIO_SMS_NUMBER").unwrap(), db)
          .await
      } else {
        Ok(())
      }
    })
    .map(|_| ())
  }
}
