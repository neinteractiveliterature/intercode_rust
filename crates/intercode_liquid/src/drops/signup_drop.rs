use intercode_entities::signups;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignupDrop {
  id: i64,
}

impl SignupDrop {
  pub fn new(signup: &signups::Model) -> Self {
    SignupDrop { id: signup.id }
  }
}
