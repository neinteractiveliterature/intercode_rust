use intercode_entities::{signups, user_con_profiles, users};
use intercode_inflector::IntercodeInflector;
use serde::{Deserialize, Serialize};

use super::SignupDrop;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserConProfileDrop<'a> {
  id: i64,
  first_name: &'a str,
  last_name: &'a str,
  privileges: Vec<String>,
  signups: Vec<SignupDrop>,
}

impl<'a> UserConProfileDrop<'a> {
  pub fn new(
    user_con_profile: &'a user_con_profiles::Model,
    user: &'a users::Model,
    signups: Box<dyn Iterator<Item = signups::Model>>,
  ) -> Self {
    let inflector = IntercodeInflector::new();

    UserConProfileDrop {
      id: user_con_profile.id,
      first_name: user_con_profile.first_name.as_str(),
      last_name: user_con_profile.last_name.as_str(),
      privileges: user
        .privileges()
        .iter()
        .map(|priv_name| inflector.humanize(priv_name))
        .collect(),
      signups: signups.map(|signup| SignupDrop::new(&signup)).collect(),
    }
  }
}
