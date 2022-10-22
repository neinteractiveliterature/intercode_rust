use intercode_entities::users;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use seawater::model_backed_drop;

use super::drop_context::DropContext;

model_backed_drop!(UserDrop, users::Model, DropContext);

#[liquid_drop_impl(i64)]
impl UserDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  pub fn email(&self) -> &str {
    self.model.email.as_str()
  }

  pub fn privileges(&self) -> Vec<&str> {
    self.model.privileges()
  }
}
