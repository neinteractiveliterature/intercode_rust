use async_graphql::*;
use intercode_entities::rooms;

use crate::model_backed_type;
model_backed_type!(RoomType, rooms::Model);

#[Object]
impl RoomType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }
}
