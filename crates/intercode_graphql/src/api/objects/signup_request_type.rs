use async_graphql::*;
use intercode_entities::signup_requests;

use crate::model_backed_type;

model_backed_type!(SignupRequestType, signup_requests::Model);

#[Object(name = "SignupRequest")]
impl SignupRequestType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn requested_bucket_key(&self) -> Option<&str> {
    self.model.requested_bucket_key.as_deref()
  }

  async fn state(&self) -> &str {
    &self.model.state
  }
}
