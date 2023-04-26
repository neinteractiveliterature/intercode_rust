use async_graphql::*;
use intercode_entities::order_entries;
use seawater::loaders::ExpectModel;

use crate::{
  model_backed_type, presenters::order_summary_presenter::load_and_describe_order_entry, QueryData,
};

use super::{money_type::MoneyType, ModelBackedType, OrderType, ProductType, ProductVariantType};
model_backed_type!(OrderEntryType, order_entries::Model);

#[Object(name = "OrderEntry")]
impl OrderEntryType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "describe_products")]
  async fn describe_products(&self, ctx: &Context<'_>) -> Result<String> {
    load_and_describe_order_entry(&self.model, ctx, false).await
  }

  async fn order(&self, ctx: &Context<'_>) -> Result<OrderType> {
    let loader_result = ctx
      .data::<QueryData>()?
      .loaders()
      .order_entry_order()
      .load_one(self.model.id)
      .await?;

    Ok(OrderType::new(loader_result.expect_one()?.clone()))
  }

  #[graphql(name = "price_per_item")]
  async fn price_per_item(&self) -> Option<MoneyType<'_>> {
    MoneyType::from_cents_and_currency(
      self.model.price_per_item_cents,
      self.model.price_per_item_currency.as_deref(),
    )
  }

  async fn product(&self, ctx: &Context<'_>) -> Result<ProductType> {
    let product_result = ctx
      .data::<QueryData>()?
      .loaders()
      .order_entry_product()
      .load_one(self.model.id)
      .await?;

    Ok(ProductType::new(product_result.expect_one()?.clone()))
  }

  #[graphql(name = "product_variant")]
  async fn product_variant(&self, ctx: &Context<'_>) -> Result<Option<ProductVariantType>> {
    let product_variant_result = ctx
      .data::<QueryData>()?
      .loaders()
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

  async fn quantity(&self) -> Option<i32> {
    self.model.quantity
  }
}
