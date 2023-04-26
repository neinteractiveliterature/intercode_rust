use std::collections::HashMap;

use async_graphql::*;
use intercode_entities::products;
use intercode_liquid::render_markdown;
use seawater::loaders::{ExpectModel, ExpectModels};

use crate::{
  api::scalars::JsonScalar, load_one_by_model_id, loader_result_to_many,
  loader_result_to_optional_single, model_backed_type,
};

use super::{
  active_storage_attachment_type::ActiveStorageAttachmentType,
  pricing_structure_type::PricingStructureType, ModelBackedType, ProductVariantType,
  TicketTypeType,
};
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

  #[graphql(name = "description_html")]
  async fn description_html(&self) -> String {
    render_markdown(
      self.model.description.as_deref().unwrap_or(""),
      &HashMap::default(),
    )
  }

  async fn image(&self, ctx: &Context<'_>) -> Result<Option<ActiveStorageAttachmentType>> {
    let loader_result = load_one_by_model_id!(product_image, ctx, self)?;

    Ok(
      loader_result
        .and_then(|blobs| blobs.get(0).cloned())
        .map(ActiveStorageAttachmentType::new),
    )
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
    let loader_result = load_one_by_model_id!(product_product_variants, ctx, self)?;

    Ok(loader_result_to_many!(loader_result, ProductVariantType))
  }

  #[graphql(name = "provides_ticket_type")]
  async fn provides_ticket_type(&self, ctx: &Context<'_>) -> Result<Option<TicketTypeType>> {
    let loader_result = load_one_by_model_id!(product_provides_ticket_type, ctx, self)?;

    Ok(loader_result_to_optional_single!(
      loader_result,
      TicketTypeType
    ))
  }
}
