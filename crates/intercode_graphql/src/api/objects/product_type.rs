use async_graphql::*;
use intercode_entities::products;
use seawater::loaders::ExpectModels;

use crate::{api::scalars::JsonScalar, model_backed_type, QueryData};

use super::{pricing_structure_type::PricingStructureType, ModelBackedType, ProductVariantType};
model_backed_type!(ProductType, products::Model);

#[Object(name = "Product")]
impl ProductType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn available(&self) -> bool {
    self.model.available.unwrap_or(false)
  }

  async fn description(&self) -> Option<&str> {
    self.model.description.as_deref()
  }

  async fn name(&self) -> &Option<String> {
    &self.model.name
  }

  #[graphql(name = "payment_options")]
  async fn payment_options(&self) -> Option<JsonScalar> {
    self.model.payment_options.clone().map(JsonScalar)
  }

  #[graphql(name = "pricing_structure")]
  async fn pricing_structure(&self) -> Result<PricingStructureType> {
    Ok(PricingStructureType::new(serde_json::from_value(
      self.model.pricing_structure.clone(),
    )?))
  }

  #[graphql(name = "product_variants")]
  async fn product_variants(&self, ctx: &Context<'_>) -> Result<Vec<ProductVariantType>> {
    let loader_result = ctx
      .data::<QueryData>()?
      .loaders()
      .product_product_variants()
      .load_one(self.model.id)
      .await?;

    Ok(
      loader_result
        .expect_models()?
        .iter()
        .map(|model| ProductVariantType::new(model.clone()))
        .collect(),
    )
  }
}
