use intercode_entities::{links::SignupToEvent, signups};
use liquid::model::DateTime;
use seawater::liquid_drop_impl;
use seawater::{belongs_to_linked, belongs_to_related, model_backed_drop, ModelBackedDrop};

use super::{drop_context::DropContext, EventDrop, RunDrop, UserConProfileDrop};

model_backed_drop!(SignupDrop, signups::Model, DropContext);

#[belongs_to_related(run, RunDrop, serialize = true, eager_load(event))]
#[belongs_to_linked(event, EventDrop, SignupToEvent, serialize = true, eager_load(runs))]
#[belongs_to_related(user_con_profile, UserConProfileDrop)]
#[liquid_drop_impl(i64, DropContext)]
impl SignupDrop {
  fn id(&self) -> i64 {
    self.model.id
  }

  async fn team_member(&self) -> bool {
    let event = self.event().await.get_inner_cloned().unwrap();
    let team_member_profiles = event
      .team_member_user_con_profiles()
      .await
      .get_inner_cloned()
      .unwrap_or_default();
    team_member_profiles
      .iter()
      .any(|ucp| ucp.get_model().id == self.get_model().user_con_profile_id)
  }

  async fn ends_at(&self) -> Option<DateTime> {
    let run = self.run().await.get_inner_cloned().unwrap();
    run.ends_at().await.get_inner_cloned()
  }

  fn state(&self) -> &str {
    &self.model.state
  }

  async fn starts_at(&self) -> Option<DateTime> {
    self
      .run()
      .await
      .get_inner_cloned()
      .unwrap()
      .starts_at()
      .await
      .get_inner_cloned()
  }
}
