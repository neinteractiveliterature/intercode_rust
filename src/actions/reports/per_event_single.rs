use std::collections::HashMap;

use askama::Template;
use intercode_entities::{
  events, rooms, runs, signups, user_con_profiles, RegistrationPolicy, SlotCount, UserNames,
};
use intercode_graphql::presenters::signup_count_presenter::RunSignupCounts;
use intercode_inflector::IntercodeInflector;

use crate::actions::reports::filters;

use super::event_run_helpers::EventRunHelpers;

#[derive(Template)]
#[template(path = "reports/per_event_single.html.j2")]
pub struct PerEventSingleTemplate {
  inflector: IntercodeInflector,
  runs_by_id: HashMap<i64, runs::Model>,
  rooms_by_run_id: HashMap<i64, Vec<rooms::Model>>,
  events_by_id: HashMap<i64, events::Model>,
  runs_by_event_id: HashMap<i64, Vec<runs::Model>>,
  event: events::Model,
  team_member_name_by_event_category_id: HashMap<i64, String>,
  team_member_profiles_by_event_id: HashMap<i64, Vec<user_con_profiles::Model>>,
  run_signup_counts_by_run_id: HashMap<i64, RunSignupCounts>,
  signups_by_run_id: HashMap<i64, Vec<signups::Model>>,
  user_con_profiles_by_id: HashMap<i64, user_con_profiles::Model>,
}

impl EventRunHelpers for PerEventSingleTemplate {
  fn get_run_by_id(&self, run_id: &i64) -> Option<&runs::Model> {
    self.runs_by_id.get(run_id)
  }

  fn get_event_by_id(&self, event_id: &i64) -> Option<&events::Model> {
    self.events_by_id.get(event_id)
  }

  fn get_rooms_for_run_id(&self, run_id: &i64) -> Option<&Vec<rooms::Model>> {
    self.rooms_by_run_id.get(run_id)
  }
}

impl PerEventSingleTemplate {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    runs_by_id: HashMap<i64, runs::Model>,
    rooms_by_run_id: HashMap<i64, Vec<rooms::Model>>,
    events_by_id: HashMap<i64, events::Model>,
    runs_by_event_id: HashMap<i64, Vec<runs::Model>>,
    event: events::Model,
    team_member_name_by_event_category_id: HashMap<i64, String>,
    team_member_profiles_by_event_id: HashMap<i64, Vec<user_con_profiles::Model>>,
    run_signup_counts_by_run_id: HashMap<i64, RunSignupCounts>,
    signups_by_run_id: HashMap<i64, Vec<signups::Model>>,
    user_con_profiles_by_id: HashMap<i64, user_con_profiles::Model>,
  ) -> Self {
    Self {
      inflector: IntercodeInflector::new(),
      runs_by_id,
      rooms_by_run_id,
      events_by_id,
      runs_by_event_id,
      event,
      team_member_name_by_event_category_id,
      team_member_profiles_by_event_id,
      run_signup_counts_by_run_id,
      signups_by_run_id,
      user_con_profiles_by_id,
    }
  }

  fn event_registration_policy(&self, event: &events::Model) -> RegistrationPolicy {
    event
      .registration_policy
      .clone()
      .map(|json| serde_json::from_value::<RegistrationPolicy>(json).unwrap_or_default())
      .unwrap_or_default()
  }

  fn plural_team_member_name_for_event(&self, event: &events::Model) -> String {
    let team_member_name = self
      .team_member_name_by_event_category_id
      .get(&event.event_category_id)
      .cloned()
      .unwrap_or("team member".to_string());

    self
      .inflector
      .pluralize(&self.inflector.titleize(&team_member_name))
  }

  fn team_member_profiles_for_event(&self, event: &events::Model) -> &[user_con_profiles::Model] {
    self
      .team_member_profiles_by_event_id
      .get(&event.id)
      .map(|vec| vec.as_slice())
      .unwrap_or_default()
  }

  fn run_signup_counts_for_run(&self, run: &runs::Model) -> RunSignupCounts {
    self
      .run_signup_counts_by_run_id
      .get(&run.id)
      .cloned()
      .unwrap_or_default()
  }

  fn signed_up_user_con_profiles_with_state_for_run(
    &self,
    run: &runs::Model,
    state: &str,
  ) -> Vec<(signups::Model, user_con_profiles::Model)> {
    let signups_by_user_con_profile_id = self
      .signups_by_run_id
      .get(&run.id)
      .cloned()
      .unwrap_or_default()
      .into_iter()
      .filter(|signup| signup.state == state)
      .map(|signup| (signup.user_con_profile_id, signup))
      .collect::<HashMap<_, _>>();

    let mut user_con_profiles = signups_by_user_con_profile_id
      .keys()
      .filter_map(|user_con_profile_id| self.user_con_profiles_by_id.get(user_con_profile_id))
      .cloned()
      .collect::<Vec<_>>();

    user_con_profiles.sort_by_cached_key(|ucp| ucp.name_inverted());

    user_con_profiles
      .into_iter()
      .map(|ucp| {
        (
          signups_by_user_con_profile_id.get(&ucp.id).unwrap().clone(),
          ucp,
        )
      })
      .collect()
  }

  fn bucket_name_for_key(&self, bucket_key: &str) -> String {
    self
      .event
      .registration_policy
      .as_ref()
      .map(|json| serde_json::from_value::<RegistrationPolicy>(json.clone()).unwrap())
      .and_then(|policy| {
        policy
          .bucket_with_key(bucket_key)
          .map(|bucket| bucket.name.clone())
      })
      .unwrap_or_else(|| bucket_key.to_string())
  }

  fn requested_bucket_name_for_signup(&self, signup: &signups::Model) -> String {
    match signup.requested_bucket_key.as_deref() {
      Some(requested_bucket_key) => self.bucket_name_for_key(requested_bucket_key),
      None => "No preference".to_string(),
    }
  }

  fn available_slot_count(&self, run: &runs::Model) -> SlotCount {
    let event = self.events_by_id.get(&run.event_id);
    let registration_policy = event
      .map(|event| self.event_registration_policy(event))
      .unwrap_or_default();
    let signups = self
      .signups_by_run_id
      .get(&run.id)
      .cloned()
      .unwrap_or_default();
    let signup_refs = signups.iter().collect::<Vec<_>>();
    registration_policy
      .all_buckets()
      .fold(SlotCount::Limited(0), |acc, bucket| {
        acc
          + bucket
            .available_slots(&signup_refs)
            .unwrap_or(SlotCount::Limited(0))
      })
  }
}
