use async_graphql::*;
use intercode_entities::signups;

use crate::model_backed_type;

model_backed_type!(SignupType, signups::Model);

#[Object(name = "Signup")]
impl SignupType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn state(&self) -> &str {
    &self.model.state
  }
}
