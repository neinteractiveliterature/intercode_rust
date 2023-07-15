use async_graphql::*;
use intercode_entities::orders;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_store::partial_objects::OrderStoreFields;

use super::{OrderEntryType, UserConProfileType};

model_backed_type!(OrderGlueFields, orders::Model);

#[Object]
impl OrderGlueFields {
  #[graphql(name = "order_entries")]
  pub async fn order_entries(&self, ctx: &Context<'_>) -> Result<Vec<OrderEntryType>, Error> {
    OrderStoreFields::from_type(self.clone())
      .order_entries(ctx)
      .await
      .map(|items| items.into_iter().map(OrderEntryType::from_type).collect())
  }

  #[graphql(name = "user_con_profile")]
  pub async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    OrderStoreFields::from_type(self.clone())
      .user_con_profile(ctx)
      .await
      .map(UserConProfileType::from_type)
  }
}

#[derive(MergedObject)]
#[graphql(name = "OrderType")]
pub struct OrderType(OrderGlueFields, OrderStoreFields);

impl ModelBackedType for OrderType {
  type Model = orders::Model;

  fn new(model: Self::Model) -> Self {
    Self(
      OrderGlueFields::new(model.clone()),
      OrderStoreFields::new(model),
    )
  }

  fn get_model(&self) -> &Self::Model {
    self.0.get_model()
  }

  fn into_model(self) -> Self::Model {
    self.0.into_model()
  }
}
