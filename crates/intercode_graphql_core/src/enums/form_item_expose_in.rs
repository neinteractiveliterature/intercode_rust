use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum FormItemExposeIn {
  #[graphql(name = "event_catalog")]
  EventCatalog,
  #[graphql(name = "schedule_popup")]
  SchedulePopup,
}
