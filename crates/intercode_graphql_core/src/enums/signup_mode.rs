use async_graphql::Enum;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum SignupMode {
  /// Attendees can sign themselves up for events
  #[graphql(name = "self_service")]
  SelfService,

  /// Attendees can request signups and signup changes but con staff must approve them
  #[graphql(name = "moderated")]
  Moderated,
}

impl TryFrom<&str> for SignupMode {
  type Error = async_graphql::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "self_service" => Ok(SignupMode::SelfService),
      "moderated" => Ok(SignupMode::Moderated),
      _ => Err(Self::Error::new(format!("Unknown signup mode: {}", value))),
    }
  }
}
