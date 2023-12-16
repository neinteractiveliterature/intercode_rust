use std::sync::Arc;

use async_graphql::*;
use intercode_entities::order_entries;
use intercode_graphql_core::{model_backed_type, objects::MoneyType, ModelBackedType};
use intercode_graphql_loaders::LoaderManager;
use seawater::loaders::ExpectModel;

use crate::{
  objects::{ProductType, ProductVariantType},
  order_summary_presenter::load_and_describe_order_entry,
};

use super::OrderStoreFields;

model_backed_type!(OrderEntryStoreFields, order_entries::Model);

impl OrderEntryStoreFields {
  pub async fn order(&self, ctx: &Context<'_>) -> Result<OrderStoreFields> {
    let loader_result = ctx
      .data::<Arc<LoaderManager>>()?
      .order_entry_order()
      .load_one(self.model.id)
      .await?;

    Ok(OrderStoreFields::new(loader_result.expect_one()?.clone()))
  }
}

#[Object]
impl OrderEntryStoreFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "describe_products")]
  async fn describe_products(&self, ctx: &Context<'_>) -> Result<String> {
    load_and_describe_order_entry(&self.model, ctx, false).await
  }

  #[graphql(name = "price_per_item")]
  async fn price_per_item(&self) -> MoneyType<'_> {
    MoneyType::from_cents_and_currency(
      self.model.price_per_item_cents,
      self.model.price_per_item_currency.as_deref(),
    )
    .unwrap_or_default()
  }

  async fn product(&self, ctx: &Context<'_>) -> Result<ProductType> {
    let product_result = ctx
      .data::<Arc<LoaderManager>>()?
      .order_entry_product()
      .load_one(self.model.id)
      .await?;

    Ok(ProductType::new(product_result.expect_one()?.clone()))
  }

  #[graphql(name = "product_variant")]
  async fn product_variant(&self, ctx: &Context<'_>) -> Result<Option<ProductVariantType>> {
    let product_variant_result = ctx
      .data::<Arc<LoaderManager>>()?
      .order_entry_product_variant()
      .load_one(self.model.id)
      .await?;

    Ok(
      product_variant_result
        .try_one()
        .cloned()
        .map(ProductVariantType::new),
    )
  }

  async fn quantity(&self) -> i32 {
    self.model.quantity.unwrap_or_default()
  }
}
