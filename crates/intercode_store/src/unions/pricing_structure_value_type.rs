use async_graphql::Union;
use chrono::Utc;
use intercode_graphql_core::objects::{MoneyType, ScheduledMoneyValueType};

use crate::objects::PayWhatYouWantValueType;

#[derive(Union)]
#[graphql(name = "PricingStructureValue")]
pub enum PricingStructureValueType {
  Fixed(MoneyType<'static>),
  ScheduledValue(ScheduledMoneyValueType<Utc>),
  PayWhatYouWant(PayWhatYouWantValueType),
}
