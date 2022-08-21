use intercode_entities::users;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};

#[liquid_drop_struct]
pub struct UserDrop {
  user: users::Model,
}

#[liquid_drop_impl]
impl UserDrop {
  pub fn new(user: users::Model) -> Self {
    UserDrop { user }
  }

  fn id(&self) -> i64 {
    self.user.id
  }

  pub fn email(&self) -> &str {
    self.user.email.as_str()
  }

  pub fn privileges(&self) -> Vec<&str> {
    self.user.privileges()
  }
}
