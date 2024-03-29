use std::sync::Arc;

use async_graphql::*;
use intercode_cms::CmsRenderingContext;
use intercode_entities::products;
use intercode_graphql_core::{
  liquid_renderer::LiquidRenderer, load_one_by_model_id, loader_result_to_many,
  loader_result_to_optional_single, model_backed_type, objects::ActiveStorageAttachmentType,
  query_data::QueryData, ModelBackedType,
};
use intercode_graphql_loaders::{
  order_quantity_by_status_loader::OrderQuantityByStatusType, LoaderManager,
};

use crate::objects::PricingStructureType;

use super::{ProductVariantStoreFields, TicketTypeStoreFields};

model_backed_type!(ProductStoreFields, products::Model);

impl ProductStoreFields {
  pub async fn product_variants(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<ProductVariantStoreFields>> {
    let loader_result = load_one_by_model_id!(product_product_variants, ctx, self)?;

    Ok(loader_result_to_many!(
      loader_result,
      ProductVariantStoreFields
    ))
  }

  pub async fn provides_ticket_type(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<TicketTypeStoreFields>> {
    let loader_result = load_one_by_model_id!(product_provides_ticket_type, ctx, self)?;

    Ok(loader_result_to_optional_single!(
      loader_result,
      TicketTypeStoreFields
    ))
  }
}

#[Object]
impl ProductStoreFields {
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
  async fn description_html(&self, ctx: &Context<'_>) -> Result<String> {
    let query_data = ctx.data::<QueryData>()?;
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;
    let cms_rendering_context =
      CmsRenderingContext::new(liquid::object!({}), query_data, liquid_renderer.as_ref());

    cms_rendering_context
      .render_liquid(self.model.description.as_deref().unwrap_or(""), None)
      .await
  }

  async fn image(&self, ctx: &Context<'_>) -> Result<Option<ActiveStorageAttachmentType>> {
    let loader_result = load_one_by_model_id!(product_image, ctx, self)?;

    Ok(
      loader_result
        .and_then(|blobs| blobs.get(0).cloned())
        .map(ActiveStorageAttachmentType::new),
    )
  }

  async fn name(&self) -> &str {
    self.model.name.as_deref().unwrap_or_default()
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
}
