use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum SiteMode {
  /// Site behaves as a convention with multiple events
  #[graphql(name = "convention")]
  Convention,

  /// Site behaves as a single standalone event
  #[graphql(name = "single_event")]
  SingleEvent,

  /// Site behaves as a series of standalone events
  #[graphql(name = "event_series")]
  EventSeries,
}
