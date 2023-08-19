use crate::NotificationDestination;

pub struct RenderedEmailNotification {
  pub subject: String,
  pub body_html: String,
  pub body_text: String,
}

pub enum RenderedNotificationContent {
  Email(RenderedEmailNotification),
  Sms(String),
}

pub struct RenderedNotification {
  pub destinations: Vec<NotificationDestination>,
  pub content: RenderedNotificationContent,
}
