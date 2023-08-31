mod config;
mod notification_destination;
mod notifier;
mod notifier_preview;
pub mod partial_objects;
mod rendered_notification;
pub mod signup_requests;

use std::{env, sync::Arc};

pub use config::*;
pub use notification_destination::*;
pub use notifier::*;
pub use notifier_preview::*;
use once_cell::sync::Lazy;
pub use rendered_notification::*;
use tracing::log::*;

static TWILIO_CLIENT: Lazy<Arc<twilio::Client>> = Lazy::new(|| {
  let sid = env::var("TWILIO_ACCOUNT_SID");
  let auth_token = env::var("TWILIO_AUTH_TOKEN");
  let (sid, auth_token) = match (sid, auth_token) {
    (Ok(sid), Ok(auth_token)) => (sid, auth_token),
    _ => {
      warn!(
        "Could not get TWILIO_ACCOUNT_SID and/or TWILIO_AUTH_TOKEN from environment, Twilio API operations will fail"
      );
      ("".to_string(), "".to_string())
    }
  };
  Arc::new(twilio::Client::new(&sid, &auth_token))
});

pub fn inject_request_data(req: async_graphql::BatchRequest) -> async_graphql::BatchRequest {
  req.data::<Arc<twilio::Client>>(TWILIO_CLIENT.clone())
}
