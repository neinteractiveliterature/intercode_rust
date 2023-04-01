use async_graphql::*;
use intercode_entities::product_variants;

use crate::model_backed_type;
model_backed_type!(ProductVariantType, product_variants::Model);

#[Object(name = "ProductVariant")]
impl ProductVariantType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn description(&self) -> Option<&str> {
    self.model.description.as_deref()
  }

  async fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }
}
