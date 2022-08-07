use intercode_entities::event_categories;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};

#[liquid_drop_struct]
pub struct EventCategoryDrop {
  event_category: event_categories::Model,
}

#[liquid_drop_impl]
impl EventCategoryDrop {
  pub fn new(event_category: event_categories::Model) -> Self {
    EventCategoryDrop { event_category }
  }

  fn id(&self) -> i64 {
    self.event_category.id
  }

  fn name(&self) -> &str {
    &self.event_category.name
  }
}
