use async_graphql::Enum;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum TimezoneMode {
  /// Display dates and times using convention’s local time zone
  #[graphql(name = "convention_local")]
  ConventionLocal,

  /// Display dates and times using user’s local time zone
  #[graphql(name = "user_local")]
  UserLocal,
}

impl TryFrom<&str> for TimezoneMode {
  type Error = async_graphql::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "convention_local" => Ok(TimezoneMode::ConventionLocal),
      "user_local" => Ok(TimezoneMode::UserLocal),
      _ => Err(Self::Error::new(format!(
        "Unknown timezone mode: {}",
        value
      ))),
    }
  }
}
