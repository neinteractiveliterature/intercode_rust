use async_graphql::Union;
use chrono::Utc;

use crate::api::objects::{MoneyType, PayWhatYouWantValueType, ScheduledMoneyValueType};

#[derive(Union)]
#[graphql(name = "PricingStructureValue")]
pub enum PricingStructureValueType {
  Fixed(MoneyType<'static>),
  ScheduledValue(ScheduledMoneyValueType<Utc>),
  PayWhatYouWant(PayWhatYouWantValueType),
}
