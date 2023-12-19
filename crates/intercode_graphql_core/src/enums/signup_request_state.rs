use async_graphql::Enum;
use strum::{EnumString, IntoStaticStr};

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum SignupRequestState {
  /// The request has not yet been reviewed by a moderator
  #[graphql(name = "pending")]
  Pending,

  /// The request has been accepted and the requester has been signed up (see the result_signup
  /// field for the actual signup)
  #[graphql(name = "accepted")]
  Accepted,

  /// The request has been rejected and the requester has not been signed up
  #[graphql(name = "rejected")]
  Rejected,

  /// The requester withdrew their request before it was accepted or rejected
  #[graphql(name = "withdrawn")]
  Withdrawn,
}
