use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ShowSchedule {
  #[graphql(name = "no")]
  No,
  #[graphql(name = "priv")]
  Priv,
  #[graphql(name = "gms")]
  GMs,
  #[graphql(name = "yes")]
  Yes,
}
