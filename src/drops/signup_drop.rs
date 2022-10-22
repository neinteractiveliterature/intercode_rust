use intercode_entities::signups;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use seawater::model_backed_drop;

use super::drop_context::DropContext;

model_backed_drop!(SignupDrop, signups::Model, DropContext);

#[liquid_drop_impl(i64)]
impl SignupDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  fn team_member(&self) -> bool {
    // TODO
    false
  }
}
