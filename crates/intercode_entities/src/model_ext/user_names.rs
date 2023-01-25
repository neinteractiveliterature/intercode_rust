pub trait UserNames {
  fn get_first_name(&self) -> &str;
  fn get_last_name(&self) -> &str;

  fn name_without_nickname(&self) -> String {
    format!("{} {}", self.get_first_name(), self.get_last_name())
      .trim()
      .to_string()
  }

  fn name_inverted(&self) -> String {
    format!("{}, {}", self.get_last_name(), self.get_first_name())
      .trim()
      .to_string()
  }
}
