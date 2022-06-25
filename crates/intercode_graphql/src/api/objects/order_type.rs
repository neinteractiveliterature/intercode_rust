use async_graphql::*;
use intercode_entities::orders;

use crate::model_backed_type;
model_backed_type!(OrderType, orders::Model);

#[Object]
impl OrderType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }
}
