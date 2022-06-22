use async_graphql::Enum;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
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

impl TryFrom<&str> for SiteMode {
  type Error = async_graphql::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "convention" => Ok(SiteMode::Convention),
      "single_event" => Ok(SiteMode::SingleEvent),
      "event_series" => Ok(SiteMode::EventSeries),
      _ => Err(Self::Error::new(format!("Unknown site mode: {}", value))),
    }
  }
}
