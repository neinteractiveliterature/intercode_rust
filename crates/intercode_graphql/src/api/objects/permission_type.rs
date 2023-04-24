use async_graphql::*;
use intercode_entities::permissions;

use crate::model_backed_type;
model_backed_type!(PermissionType, permissions::Model);

#[Object(name = "Permission")]
impl PermissionType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn permission(&self) -> &str {
    &self.model.permission
  }
}
