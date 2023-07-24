use async_graphql::*;
use intercode_entities::order_entries;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_store::partial_objects::OrderEntryStoreFields;

use super::OrderType;

model_backed_type!(OrderEntryGlueFields, order_entries::Model);

#[Object]
impl OrderEntryGlueFields {
  pub async fn order(&self, ctx: &Context<'_>) -> Result<OrderType> {
    OrderEntryStoreFields::from_type(self.clone())
      .order(ctx)
      .await
      .map(OrderType::from_type)
  }
}

#[derive(MergedObject)]
#[graphql(name = "OrderEntryType")]
pub struct OrderEntryType(OrderEntryGlueFields, OrderEntryStoreFields);

impl ModelBackedType for OrderEntryType {
  type Model = order_entries::Model;

  fn new(model: Self::Model) -> Self {
    Self(
      OrderEntryGlueFields::new(model.clone()),
      OrderEntryStoreFields::new(model),
    )
  }

  fn get_model(&self) -> &Self::Model {
    self.0.get_model()
  }

  fn into_model(self) -> Self::Model {
    self.0.into_model()
  }
}
