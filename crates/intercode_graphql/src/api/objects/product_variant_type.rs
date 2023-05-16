use super::{
  active_storage_attachment_type::ActiveStorageAttachmentType,
  pricing_structure_type::PricingStructureType, OrderQuantityByStatusType,
};
use crate::{
  api::objects::model_backed_type::ModelBackedType, load_one_by_model_id, model_backed_type,
  QueryData,
};
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

  async fn image(&self, ctx: &Context<'_>) -> Result<Option<ActiveStorageAttachmentType>> {
    let loader_result = load_one_by_model_id!(product_variant_image, ctx, self)?;

    Ok(
      loader_result
        .and_then(|blobs| blobs.get(0).cloned())
        .map(ActiveStorageAttachmentType::new),
    )
  }

  async fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }

  #[graphql(name = "order_quantities_by_status")]
  async fn order_quantities_by_status(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<OrderQuantityByStatusType>> {
    Ok(
      ctx
        .data::<QueryData>()?
        .loaders()
        .product_variant_order_quantity_by_status
        .load_one(self.model.id)
        .await?
        .unwrap_or_default(),
    )
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
