use crate::staff_positions;
use async_graphql::*;

use crate::model_backed_type;
model_backed_type!(StaffPositionType, staff_positions::Model);

#[Object]
impl StaffPositionType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }
}
