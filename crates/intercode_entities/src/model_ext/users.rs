use super::UserNames;
use crate::users;

impl UserNames for users::Model {
  fn get_first_name(&self) -> &str {
    self.first_name.as_str()
  }

  fn get_last_name(&self) -> &str {
    self.last_name.as_str()
  }
}
