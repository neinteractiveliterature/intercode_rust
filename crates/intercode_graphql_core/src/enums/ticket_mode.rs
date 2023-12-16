use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
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
