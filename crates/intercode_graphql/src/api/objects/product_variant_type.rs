use super::pricing_structure_type::PricingStructureType;
use crate::model_backed_type;
use async_graphql::*;
use intercode_entities::product_variants;

model_backed_type!(ProductVariantType, product_variants::Model);

#[Object(name = "ProductVariant")]
impl ProductVariantType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn description(&self) -> Option<&str> {
    self.model.description.as_deref()
  }

  async fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }

  #[graphql(name = "override_pricing_structure")]
  async fn override_pricing_structure(&self) -> Result<Option<PricingStructureType>> {
    Ok(
      self
        .model
        .override_pricing_structure
        .clone()
        .map(|ps| serde_json::from_value(ps).map(PricingStructureType::new))
        .transpose()?,
    )
  }

  async fn position(&self) -> Option<i32> {
    self.model.position
  }
}
