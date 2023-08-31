use async_graphql::{futures_util::future::try_join_all, Error};
use aws_sdk_sesv2::types::Destination;
use seawater::ConnectionWrapper;
use twilio::OutboundMessage;

use crate::{NotificationDestination, TWILIO_CLIENT};

pub struct RenderedNotification {
  pub destinations: Vec<NotificationDestination>,
  pub subject: String,
  pub body_html: Option<String>,
  pub body_text: Option<String>,
  pub body_sms: Option<String>,
}

impl RenderedNotification {
  pub fn body_text(&self) -> String {
    match &self.body_text {
      Some(body_text) => body_text.to_owned(),
      // TODO: strip HTML tags
      None => self.body_html.to_owned().unwrap_or_default(),
    }
  }

  pub async fn send_email(&self, from_email: &str, db: &ConnectionWrapper) -> Result<(), Error> {
    let emails = NotificationDestination::load_emails(self.destinations.iter(), db).await?;
    intercode_email::send_email(
      from_email,
      Destination::builder()
        .to_addresses(emails.join(", "))
        .build(),
      &self.subject,
      self.body_html.as_deref(),
      self.body_text.as_deref(),
    )
    .await?;

    Ok(())
  }

  pub async fn send_sms(&self, from_sms: &str, db: &ConnectionWrapper) -> Result<(), Error> {
    let sms_numbers =
      NotificationDestination::load_sms_numbers(self.destinations.iter(), db).await?;

    let body = self.body_sms.clone().unwrap_or_else(|| self.body_text());

    try_join_all(sms_numbers.iter().map(|sms_number| async {
      let sms_number = phonenumber::parse(None, sms_number.clone()).map_err(Error::from)?;
      TWILIO_CLIENT
        .send_message(OutboundMessage::new(
          from_sms,
          &sms_number
            .format()
            .mode(phonenumber::Mode::E164)
            .to_string(),
          &body,
        ))
        .await
        .map_err(Error::from)
    }))
    .await?;

    Ok(())
  }
}
