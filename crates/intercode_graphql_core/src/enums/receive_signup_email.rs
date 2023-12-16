use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ReceiveSignupEmail {
  /// Receive email for all signup activity
  AllSignups,
  /// Receive email for signup activity affecting confirmed signups
  NonWaitlistSignups,
  /// Do not receive signup email
  No,
}
