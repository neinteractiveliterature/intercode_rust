use async_graphql::*;
use intercode_entities::products;

use crate::model_backed_type;
model_backed_type!(ProductType, products::Model);

#[Object]
impl ProductType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn available(&self) -> bool {
    self.model.available.unwrap_or(false)
  }

  async fn name(&self) -> &Option<String> {
    &self.model.name
  }
}
