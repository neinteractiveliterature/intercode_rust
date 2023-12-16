use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum SignupMode {
  /// Attendees can sign themselves up for events
  #[graphql(name = "self_service")]
  SelfService,

  /// Attendees can request signups and signup changes but con staff must approve them
  #[graphql(name = "moderated")]
  Moderated,
}
