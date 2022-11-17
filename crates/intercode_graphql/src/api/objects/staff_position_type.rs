use async_graphql::*;
use intercode_entities::staff_positions;

use crate::model_backed_type;
model_backed_type!(StaffPositionType, staff_positions::Model);

#[Object(name = "StaffPosition")]
impl StaffPositionType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn name(&self) -> &Option<String> {
    &self.model.name
  }
}
