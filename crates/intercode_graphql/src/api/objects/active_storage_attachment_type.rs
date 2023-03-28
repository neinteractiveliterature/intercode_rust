use crate::model_backed_type;
use async_graphql::*;
use intercode_entities::active_storage_blobs;

model_backed_type!(ActiveStorageAttachmentType, active_storage_blobs::Model);

/// Despite the name, this actually represents an active_storage_blob model.  Whoops...
#[Object(name = "ActiveStorageAttachment")]
impl ActiveStorageAttachmentType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "byte_size")]
  async fn byte_size(&self) -> i64 {
    self.model.byte_size
  }

  #[graphql(name = "content_type")]
  async fn content_type(&self) -> Option<&str> {
    self.model.content_type.as_deref()
  }

  async fn filename(&self) -> &str {
    &self.model.filename
  }

  async fn url(&self) -> &str {
    // TODO
    "TODO"
  }
}
