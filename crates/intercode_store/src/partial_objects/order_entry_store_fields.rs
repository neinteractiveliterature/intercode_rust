use async_graphql::*;
use async_trait::async_trait;
use intercode_entities::{order_entries, orders, product_variants, products};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_optional_single, loader_result_to_required_single,
  model_backed_type, objects::MoneyType, ModelBackedType,
};

use crate::order_summary_presenter::load_and_describe_order_entry;

#[async_trait]
pub trait OrderEntryStoreExtensions
where
  Self: ModelBackedType<Model = order_entries::Model>,
{
  async fn order<T: ModelBackedType<Model = orders::Model>>(&self, ctx: &Context<'_>) -> Result<T> {
    let loader_result = load_one_by_model_id!(order_entry_order, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }

  async fn product<T: ModelBackedType<Model = products::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<T> {
    let loader_result = load_one_by_model_id!(order_entry_product, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }

  async fn product_variant<T: ModelBackedType<Model = product_variants::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<T>> {
    let loader_result = load_one_by_model_id!(order_entry_product_variant, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, T))
  }
}

model_backed_type!(OrderEntryStoreFields, order_entries::Model);

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

  async fn quantity(&self) -> i32 {
    self.model.quantity.unwrap_or_default()
  }
}
