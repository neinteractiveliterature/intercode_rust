use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TimezoneMode {
  /// Display dates and times using convention’s local time zone
  #[graphql(name = "convention_local")]
  ConventionLocal,

  /// Display dates and times using user’s local time zone
  #[graphql(name = "user_local")]
  UserLocal,
}
