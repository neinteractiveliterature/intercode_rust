use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum OrderStatus {
  /// Order has not yet been submitted
  #[graphql(name = "pending")]
  Pending,

  /// Order is submitted but not yet paid
  #[graphql(name = "unpaid")]
  Unpaid,

  /// Order has been submitted and paid
  #[graphql(name = "paid")]
  Paid,

  /// Order has been cancelled
  #[graphql(name = "cancelled")]
  Cancelled,
}
