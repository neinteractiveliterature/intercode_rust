use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum FormType {
  #[graphql(name = "event")]
  Event,
  #[graphql(name = "event_proposal")]
  EventProposal,
  #[graphql(name = "user_con_profile")]
  UserConProfile,
}
