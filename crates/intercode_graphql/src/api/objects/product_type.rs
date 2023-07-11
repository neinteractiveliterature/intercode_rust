use std::{collections::HashMap, sync::Arc};

use async_graphql::*;
use intercode_entities::products;
use intercode_graphql_loaders::{
  order_quantity_by_status_loader::OrderQuantityByStatusType, LoaderManager,
};
use intercode_liquid::render_markdown;

use crate::{
  load_one_by_model_id, loader_result_to_many, loader_result_to_optional_single, model_backed_type,
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

  #[graphql(name = "order_quantities_by_status")]
  async fn order_quantities_by_status(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<OrderQuantityByStatusType>> {
    Ok(
      ctx
        .data::<Arc<LoaderManager>>()?
        .product_order_quantity_by_status
        .load_one(self.model.id)
        .await?
        .unwrap_or_default(),
    )
  }

  #[graphql(name = "payment_options")]
  async fn payment_options(&self) -> Vec<String> {
    self
      .model
      .payment_options
      .as_ref()
      .and_then(|value| value.as_array())
      .cloned()
      .unwrap_or_default()
      .into_iter()
      .filter_map(|item| item.as_str().map(str::to_string))
      .collect()
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
