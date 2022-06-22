use async_graphql::*;
use intercode_entities::cms_layouts;

use crate::model_backed_type;
model_backed_type!(CmsLayoutType, cms_layouts::Model);

#[Object]
impl CmsLayoutType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }
}
