use async_graphql::Object;

use crate::api::enums::OrderStatus;

#[derive(Debug, Clone)]
pub struct OrderQuantityByStatusType {
  status: OrderStatus,
  quantity: i64,
}

impl OrderQuantityByStatusType {
  pub fn new(status: OrderStatus, quantity: i64) -> Self {
    Self { status, quantity }
  }
}

#[Object]
impl OrderQuantityByStatusType {
  pub async fn status(&self) -> OrderStatus {
    self.status
  }

  pub async fn quantity(&self) -> i64 {
    self.quantity
  }
}
