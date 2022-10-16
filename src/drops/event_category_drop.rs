use intercode_entities::event_categories;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use seawater::model_backed_drop;

model_backed_drop!(EventCategoryDrop, event_categories::Model);

#[liquid_drop_impl]
impl EventCategoryDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  fn name(&self) -> &str {
    &self.model.name
  }
}
