use async_graphql::*;
use intercode_entities::cms_partials;

use crate::model_backed_type;
model_backed_type!(CmsPartialType, cms_partials::Model);

#[Object(name = "CmsPartial")]
impl CmsPartialType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }
}
