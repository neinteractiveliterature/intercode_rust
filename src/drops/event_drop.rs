use intercode_entities::{events, links::EventToTeamMemberUserConProfiles};
use liquid::model::DateTime;
use seawater::liquid_drop_impl;
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
#[liquid_drop_impl(i64, DropContext)]
impl EventDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  fn author(&self) -> Option<&str> {
    self.model.author.as_deref()
  }

  fn created_at(&self) -> Option<DateTime> {
    self
      .model
      .created_at
      .and_then(naive_date_time_to_liquid_date_time)
  }

  fn email(&self) -> Option<&str> {
    self.model.email.as_deref()
  }

  pub fn length_seconds(&self) -> i32 {
    self.model.length_seconds
  }

  async fn team_member_name(&self) -> String {
    let event_category = self.event_category().await.get_inner_cloned();
    let name_future =
      event_category.map(|event_category| async move { event_category.team_member_name().await });

    let name = if let Some(name_future) = name_future {
      name_future.await
    } else {
      "team_member".to_string().into()
    };

    name.get_inner_cloned().unwrap()
  }

  fn title(&self) -> &str {
    self.model.title.as_str()
  }
}
