use async_graphql::*;
use intercode_entities::order_entries;

use crate::model_backed_type;
model_backed_type!(OrderEntryType, order_entries::Model);

#[Object(name = "OrderEntry")]
impl OrderEntryType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn quantity(&self) -> Option<i32> {
    self.model.quantity
  }
}
