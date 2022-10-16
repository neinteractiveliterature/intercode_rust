use intercode_entities::{links::StaffPositionToUserConProfiles, staff_positions};
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use seawater::{has_many_linked, model_backed_drop};

use super::UserConProfileDrop;

model_backed_drop!(StaffPositionDrop, staff_positions::Model);

#[has_many_linked(user_con_profiles, UserConProfileDrop, StaffPositionToUserConProfiles)]
#[liquid_drop_impl]
impl StaffPositionDrop {
  pub fn id(&self) -> i64 {
    self.model.id
  }

  fn email(&self) -> Option<&str> {
    self.model.email.as_deref()
  }

  fn email_link(&self) -> Option<String> {
    self
      .email()
      .map(|email| format!("<a href=\"mailto:{}\">{}</a>", email, email))
  }

  fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }
}
