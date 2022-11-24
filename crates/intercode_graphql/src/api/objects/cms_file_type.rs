use async_graphql::*;
use intercode_entities::cms_files;

use crate::model_backed_type;
model_backed_type!(CmsFileType, cms_files::Model);

#[Object(name = "CmsFile")]
impl CmsFileType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }
}
