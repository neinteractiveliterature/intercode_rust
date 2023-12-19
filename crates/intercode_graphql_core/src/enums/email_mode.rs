use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum EmailMode {
  /// Forward received emails to staff positions as configured
  #[graphql(name = "forward")]
  Forward,

  /// Forward all received staff emails to catch-all staff position
  #[graphql(name = "staff_emails_to_catch_all")]
  StaffEmailsToCatchAll,
}
