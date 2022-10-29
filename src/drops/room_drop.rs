use intercode_entities::rooms;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use seawater::model_backed_drop;

use super::drop_context::DropContext;

model_backed_drop!(RoomDrop, rooms::Model, DropContext);

#[liquid_drop_impl(i64)]
impl RoomDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  fn name(&self) -> Option<&String> {
    self.model.name.as_ref()
  }
}
