use std::collections::HashMap;

use askama::Template;
use intercode_entities::{events, rooms, runs, user_con_profiles, RegistrationPolicy};
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
  pub fn new(
    runs_by_id: HashMap<i64, runs::Model>,
    rooms_by_run_id: HashMap<i64, Vec<rooms::Model>>,
    events_by_id: HashMap<i64, events::Model>,
    runs_by_event_id: HashMap<i64, Vec<runs::Model>>,
    event: events::Model,
    team_member_name_by_event_category_id: HashMap<i64, String>,
    team_member_profiles_by_event_id: HashMap<i64, Vec<user_con_profiles::Model>>,
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
}
