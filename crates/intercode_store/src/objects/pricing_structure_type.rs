use async_graphql::Object;
use chrono::Utc;
use intercode_entities::PricingStructure;
use intercode_graphql_core::{
  enums::PricingStrategy,
  objects::{MoneyType, ScheduledMoneyValueType},
  scalars::DateScalar,
};

use crate::unions::PricingStructureValueType;

use super::PayWhatYouWantValueType;

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
  pub async fn price(&self, time: Option<DateScalar>) -> Option<MoneyType<'static>> {
    self
      .pricing_structure
      .price(time.map(|date| date.0).unwrap_or_else(Utc::now))
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
