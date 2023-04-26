use async_graphql::Enum;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum SignupState {
  /// Attendee's spot is held temporarily while the attendee finishes paying for their ticket
  #[graphql(name = "ticket_purchase_hold")]
  TicketPurchaseHold,

  /// Attendee's spot is confirmed
  #[graphql(name = "confirmed")]
  Confirmed,

  /// Attendee is on the waitlist for this event and may be pulled in automatically
  #[graphql(name = "waitlisted")]
  Waitlisted,

  /// Attendee has withdrawn from this event (and this signup is no longer valid)
  #[graphql(name = "withdrawn")]
  Withdrawn,
}

impl TryFrom<&str> for SignupState {
  type Error = async_graphql::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "confirmed" => Ok(SignupState::Confirmed),
      "ticket_purchase_hold" => Ok(SignupState::TicketPurchaseHold),
      "waitlisted" => Ok(SignupState::Waitlisted),
      "withdrawn" => Ok(SignupState::Withdrawn),
      _ => Err(Self::Error::new(format!("Unknown signup state: {}", value))),
    }
  }
}
