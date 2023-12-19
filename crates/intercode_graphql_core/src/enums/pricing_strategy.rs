use async_graphql::Enum;
use strum::EnumString;

#[derive(Enum, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum PricingStrategy {
  /// Fixed price
  #[graphql(name = "fixed")]
  Fixed,

  /// Price that changes over time
  #[graphql(name = "scheduled_value")]
  ScheduledValue,

  /// Pay-what-you-want price
  #[graphql(name = "pay_what_you_want")]
  PayWhatYouWant,
}
