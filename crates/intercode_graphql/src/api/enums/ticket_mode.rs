use async_graphql::Enum;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum TicketMode {
  /// Tickets are neither sold nor required in this convention
  #[graphql(name = "disabled")]
  Disabled,

  /// A valid ticket is required to sign up for events in this convention
  #[graphql(name = "required_for_signup")]
  RequiredForSignup,

  /// Each event in this convention sells tickets separately
  #[graphql(name = "ticket_per_event")]
  TicketPerEvent,
}

impl TryFrom<&str> for TicketMode {
  type Error = async_graphql::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "disabled" => Ok(TicketMode::Disabled),
      "required_for_signup" => Ok(TicketMode::RequiredForSignup),
      "ticket_per_event" => Ok(TicketMode::TicketPerEvent),
      _ => Err(Self::Error::new(format!("Unknown ticket mode: {}", value))),
    }
  }
}
