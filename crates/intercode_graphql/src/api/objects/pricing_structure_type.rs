use crate::api::enums::PricingStrategy;
use async_graphql::Object;
use intercode_entities::PricingStructure;

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
  #[graphql(name = "pricing_strategy")]
  pub async fn pricing_strategy(&self) -> PricingStrategy {
    match self.pricing_structure {
      PricingStructure::Fixed(_) => PricingStrategy::Fixed,
      PricingStructure::Scheduled(_) => PricingStrategy::ScheduledValue,
      PricingStructure::PayWhatYouWant(_) => PricingStrategy::PayWhatYouWant,
    }
  }
}
