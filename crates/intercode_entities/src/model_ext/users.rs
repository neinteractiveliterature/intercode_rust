use super::UserNames;
use crate::users;

impl users::Model {
  pub fn privileges(&self) -> Vec<&str> {
    if let Some(true) = self.site_admin {
      vec!["site_admin"]
    } else {
      vec![]
    }
  }
}

impl UserNames for users::Model {
  fn get_first_name(&self) -> &str {
    self.first_name.as_str()
  }

  fn get_last_name(&self) -> &str {
    self.last_name.as_str()
  }
}
