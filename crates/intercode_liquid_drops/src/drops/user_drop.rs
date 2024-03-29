use intercode_entities::users;
use seawater::liquid_drop_impl;
use seawater::model_backed_drop;

use super::drop_context::DropContext;

model_backed_drop!(UserDrop, users::Model, DropContext);

#[liquid_drop_impl(i64, DropContext)]
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
