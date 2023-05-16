use async_graphql::{resolver_utils::parse_enum, Enum};

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
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

impl TryFrom<&str> for OrderStatus {
  type Error = async_graphql::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    parse_enum(value.into())
      .map_err(|err| Self::Error::new(err.into_server_error(Default::default()).message))
  }
}
