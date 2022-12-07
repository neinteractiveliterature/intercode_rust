use intercode_entities::{links::UserConProfileToStaffPositions, user_con_profiles, UserNames};
use intercode_inflector::IntercodeInflector;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use seawater::{
  belongs_to_related, has_many_linked, has_many_related, has_one_related, model_backed_drop,
  DropError,
};

use super::{drop_context::DropContext, SignupDrop, StaffPositionDrop, TicketDrop, UserDrop};

model_backed_drop!(UserConProfileDrop, user_con_profiles::Model, DropContext);

#[has_many_related(
  signups,
  SignupDrop,
  eager_load(event, run),
  inverse = "user_con_profile"
)]
#[has_many_linked(
  staff_positions,
  StaffPositionDrop,
  UserConProfileToStaffPositions,
  serialize = true
)]
#[has_one_related(ticket, TicketDrop, serialize = true, eager_load(ticket_type))]
#[belongs_to_related(user, UserDrop, serialize = true)]
#[liquid_drop_impl(i64)]
impl UserConProfileDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  fn first_name(&self) -> &str {
    self.model.first_name.as_str()
  }

  fn ical_secret(&self) -> &str {
    self.model.ical_secret.as_str()
  }

  fn last_name(&self) -> &str {
    self.model.last_name.as_str()
  }

  fn name_without_nickname(&self) -> String {
    self.model.name_without_nickname()
  }

  async fn privileges(&self) -> Result<Vec<String>, DropError> {
    let inflector = IntercodeInflector::new();

    Ok(
      self
        .user()
        .await
        .expect_inner()
        .privileges()
        .await
        .expect_inner()
        .iter()
        .map(|priv_name| inflector.humanize(priv_name))
        .collect::<Vec<_>>(),
    )
  }
}
