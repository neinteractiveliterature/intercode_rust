use super::drop_context::DropContext;
use intercode_entities::event_categories;
use seawater::liquid_drop_impl;
use seawater::model_backed_drop;

model_backed_drop!(EventCategoryDrop, event_categories::Model, DropContext);

#[liquid_drop_impl(i64)]
impl EventCategoryDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  fn name(&self) -> &str {
    &self.model.name
  }

  fn team_member_name(&self) -> &str {
    &self.model.team_member_name
  }
}
