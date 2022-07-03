use intercode_entities::user_con_profiles;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserConProfileDrop<'a> {
  id: i64,
  first_name: &'a str,
  last_name: &'a str,
}

impl<'a> UserConProfileDrop<'a> {
  pub fn new(user_con_profile: &'a user_con_profiles::Model) -> Self {
    UserConProfileDrop {
      id: user_con_profile.id,
      first_name: user_con_profile.first_name.as_str(),
      last_name: user_con_profile.last_name.as_str(),
    }
  }
}
