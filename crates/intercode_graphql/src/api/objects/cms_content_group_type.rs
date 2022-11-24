use async_graphql::*;
use intercode_entities::cms_content_groups;

use crate::model_backed_type;
model_backed_type!(CmsContentGroupType, cms_content_groups::Model);

#[Object(name = "CmsContentGroup")]
impl CmsContentGroupType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }
}
