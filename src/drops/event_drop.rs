use intercode_entities::events;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};

#[liquid_drop_struct]
pub struct EventDrop {
  event: events::Model,
}

#[liquid_drop_impl]
impl EventDrop {
  pub fn new(event: events::Model) -> Self {
    EventDrop { event }
  }

  fn id(&self) -> i64 {
    self.event.id
  }
}
