use intercode_entities::signups;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};

#[liquid_drop_struct]
pub struct SignupDrop {
  signup: signups::Model,
}

#[liquid_drop_impl]
impl SignupDrop {
  pub fn new(signup: signups::Model) -> Self {
    SignupDrop { signup }
  }

  fn id(&self) -> i64 {
    self.signup.id
  }
}
