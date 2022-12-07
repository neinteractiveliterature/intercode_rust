use intercode_entities::{events, links::EventToTeamMemberUserConProfiles};
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use liquid::model::DateTime;
use seawater::{belongs_to_related, has_many_linked, has_many_related, model_backed_drop};

use super::{
  drop_context::DropContext, utils::naive_date_time_to_liquid_date_time, EventCategoryDrop,
  RunDrop, UserConProfileDrop,
};

model_backed_drop!(EventDrop, events::Model, DropContext);

#[belongs_to_related(event_category, EventCategoryDrop, serialize = true)]
#[has_many_related(runs, RunDrop, inverse(event), eager_load(rooms))]
#[has_many_linked(
  team_member_user_con_profiles,
  UserConProfileDrop,
  EventToTeamMemberUserConProfiles,
  serialize = true,
  eager_load(signups, staff_positions, ticket, user)
)]
#[liquid_drop_impl(i64)]
impl EventDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  fn created_at(&self) -> Option<DateTime> {
    self
      .model
      .created_at
      .and_then(naive_date_time_to_liquid_date_time)
  }

  pub fn length_seconds(&self) -> i32 {
    self.model.length_seconds
  }

  fn title(&self) -> &str {
    self.model.title.as_str()
  }
}
