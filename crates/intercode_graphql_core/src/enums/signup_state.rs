use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
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
