use intercode_entities::users;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use seawater::model_backed_drop;

model_backed_drop!(UserDrop, users::Model);

#[liquid_drop_impl]
impl UserDrop {
  pub fn id(&self) -> i64 {
    self.model.id
  }

  pub fn email(&self) -> &str {
    self.model.email.as_str()
  }

  pub fn privileges(&self) -> Vec<&str> {
    self.model.privileges()
  }
}
