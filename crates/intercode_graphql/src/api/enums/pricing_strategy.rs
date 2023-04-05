use async_graphql::Enum;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
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
