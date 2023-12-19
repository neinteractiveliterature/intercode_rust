use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum SchedulingUi {
  #[graphql(name = "regular")]
  Regular,
  #[graphql(name = "recurring")]
  Recurring,
  #[graphql(name = "single_run")]
  SingleRun,
}
