use async_graphql::Enum;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum EmailMode {
  /// Forward received emails to staff positions as configured
  #[graphql(name = "forward")]
  Forward,

  /// Forward all received staff emails to catch-all staff position
  #[graphql(name = "staff_emails_to_catch_all")]
  StaffEmailsToCatchAll,
}

impl TryFrom<&str> for EmailMode {
  type Error = async_graphql::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "forward" => Ok(EmailMode::Forward),
      "staff_emails_to_catch_all" => Ok(EmailMode::StaffEmailsToCatchAll),
      _ => Err(Self::Error::new(format!("Unknown email mode: {}", value))),
    }
  }
}
