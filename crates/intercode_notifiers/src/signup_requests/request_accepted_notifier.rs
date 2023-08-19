use async_graphql::{async_trait::async_trait, Error};
use intercode_entities::{conventions, signup_requests, user_con_profiles};
use intercode_liquid_drops::drops::{DropContext, SignupRequestDrop};
use liquid::object;
use sea_orm::{DbErr, ModelTrait};
use seawater::{ConnectionWrapper, ModelBackedDrop};

use crate::{NotificationDestination, Notifier};

pub struct RequestAcceptedNotifier {
  convention: conventions::Model,
  signup_request: signup_requests::Model,
  liquid_assigns: liquid::Object,
}

impl RequestAcceptedNotifier {
  pub fn new(
    convention: conventions::Model,
    signup_request: signup_requests::Model,
    ctx: DropContext,
  ) -> Self {
    Self {
      convention,
      signup_request: signup_request.clone(),
      liquid_assigns: object!({
        "signup_request": SignupRequestDrop::new(signup_request, ctx)
      }),
    }
  }

  pub fn with_liquid_assigns(
    convention: conventions::Model,
    signup_request: signup_requests::Model,
    liquid_assigns: liquid::Object,
  ) -> Self {
    Self {
      convention,
      signup_request,
      liquid_assigns,
    }
  }
}

#[async_trait]
impl Notifier for RequestAcceptedNotifier {
  fn get_convention(&self) -> &conventions::Model {
    &self.convention
  }

  fn get_category_key(&self) -> &str {
    "signup_requests"
  }

  fn get_event_key(&self) -> &str {
    "request_accepted"
  }

  fn get_liquid_assigns(&self) -> liquid::Object {
    self.liquid_assigns.clone()
  }

  async fn get_destinations(
    &self,
    db: &ConnectionWrapper,
  ) -> Result<Vec<NotificationDestination>, Error> {
    let user_con_profile = self
      .signup_request
      .find_related(user_con_profiles::Entity)
      .one(db)
      .await?
      .ok_or_else(|| {
        DbErr::RecordNotFound(format!(
          "UserConProfile for SignupRequest {} could not be found",
          self.signup_request.id
        ))
      })?;

    Ok(vec![NotificationDestination::UserConProfile(
      user_con_profile,
    )])
  }
}
