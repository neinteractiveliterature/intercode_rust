use crate::api::unions::PricingStructureValueType;
use async_graphql::Object;
use chrono::{DateTime, Utc};
use intercode_entities::PricingStructure;
use intercode_graphql_core::enums::PricingStrategy;

use super::{MoneyType, PayWhatYouWantValueType, ScheduledMoneyValueType};

pub struct PricingStructureType {
  pricing_structure: PricingStructure,
}

impl PricingStructureType {
  pub fn new(pricing_structure: PricingStructure) -> Self {
    Self { pricing_structure }
  }
}

#[Object(name = "PricingStructure")]
impl PricingStructureType {
  pub async fn price(&self, time: Option<DateTime<Utc>>) -> Option<MoneyType<'static>> {
    self
      .pricing_structure
      .price(time.unwrap_or_else(Utc::now))
      .map(MoneyType::new)
  }

  #[graphql(name = "pricing_strategy")]
  pub async fn pricing_strategy(&self) -> PricingStrategy {
    match self.pricing_structure {
      PricingStructure::Fixed(_) => PricingStrategy::Fixed,
      PricingStructure::Scheduled(_) => PricingStrategy::ScheduledValue,
      PricingStructure::PayWhatYouWant(_) => PricingStrategy::PayWhatYouWant,
    }
  }

  pub async fn value(&self) -> PricingStructureValueType {
    match &self.pricing_structure {
      PricingStructure::Fixed(value) => {
        PricingStructureValueType::Fixed(MoneyType::new(value.to_owned()))
      }
      PricingStructure::Scheduled(value) => {
        PricingStructureValueType::ScheduledValue(ScheduledMoneyValueType::new(value.to_owned()))
      }
      PricingStructure::PayWhatYouWant(value) => {
        PricingStructureValueType::PayWhatYouWant(PayWhatYouWantValueType::new(value.to_owned()))
      }
    }
  }
}
