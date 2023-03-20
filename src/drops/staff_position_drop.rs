use intercode_entities::{links::StaffPositionToUserConProfiles, staff_positions};
use seawater::liquid_drop_impl;
use seawater::{has_many_linked, model_backed_drop};

use super::{drop_context::DropContext, UserConProfileDrop};

model_backed_drop!(StaffPositionDrop, staff_positions::Model, DropContext);

#[has_many_linked(user_con_profiles, UserConProfileDrop, StaffPositionToUserConProfiles)]
#[liquid_drop_impl(i64, DropContext)]
impl StaffPositionDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  fn email(&self) -> Option<&str> {
    self.model.email.as_deref()
  }

  async fn email_link(&self) -> Option<String> {
    self
      .email()
      .await
      .get_inner()
      .as_option()
      .map(|email| format!("<a href=\"mailto:{}\">{}</a>", email, email))
  }

  fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }
}
