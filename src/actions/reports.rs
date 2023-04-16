use std::{
  collections::{HashMap, HashSet},
  env,
};

use askama::Template;
use axum::{
  extract::Path,
  response::{Html, IntoResponse},
};
use chrono::NaiveDateTime;
use http::StatusCode;
use intercode_entities::{
  events, rooms, rooms_runs, runs, signups, team_members, user_con_profiles, RegistrationPolicy,
  UserNames,
};
use intercode_graphql::rendering_utils::url_with_possible_host;
use itertools::Itertools;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};

use crate::middleware::QueryDataFromRequest;

mod filters {
  use chrono::NaiveDateTime;

  pub fn format_run_day_and_time(datetime: Option<NaiveDateTime>) -> Result<String, askama::Error> {
    Ok(
      datetime
        .map(|time| time.format("%a %l:%M%P").to_string())
        .unwrap_or_default(),
    )
  }
}

#[derive(Template)]
#[template(path = "reports/single_user_printable.html.j2")]
struct SingleUserPrintableTemplate {
  page_title: String,
  application_styles_js_url: String,
  convention_name: String,
  report_type: String,
  events_by_id: HashMap<i64, events::Model>,
  user_con_profile: user_con_profiles::Model,
  active_signups: Vec<signups::Model>,
  runs_by_id: HashMap<i64, runs::Model>,
  rooms_by_run_id: HashMap<i64, Vec<rooms::Model>>,
  team_member_events: Vec<events::Model>,
  runs_by_event_id: HashMap<i64, Vec<runs::Model>>,
}

impl SingleUserPrintableTemplate {
  fn run_starts_at(&self, run_id: &i64) -> Option<NaiveDateTime> {
    self.runs_by_id.get(run_id).and_then(|run| run.starts_at)
  }

  fn room_names_for_run(&self, run_id: &i64) -> String {
    let names = self
      .rooms_by_run_id
      .get(run_id)
      .map(|rooms| {
        rooms
          .iter()
          .filter_map(|room| room.name.as_deref())
          .collect::<HashSet<_>>()
      })
      .unwrap_or_default();

    let mut names = Vec::from_iter(names.into_iter());
    names.sort();
    names.join(", ")
  }

  fn title_for_run(&self, run_id: &i64) -> String {
    let Some(run) = self.runs_by_id.get(run_id) else { return "".to_string() };

    let event_title = self
      .events_by_id
      .get(&run.event_id)
      .map(|event| event.title.as_str())
      .unwrap_or_default();

    match run.title_suffix.as_deref() {
      Some(suffix) => format!("{} ({})", event_title, suffix),
      None => event_title.to_string(),
    }
  }

  fn titleized_state(&self, state: &str) -> String {
    inflector::cases::titlecase::to_title_case(state)
  }

  fn event_registration_policy(&self, event: &events::Model) -> RegistrationPolicy {
    event
      .registration_policy
      .clone()
      .map(|json| serde_json::from_value::<RegistrationPolicy>(json).unwrap_or_default())
      .unwrap_or_default()
  }
}

pub async fn single_user_printable(
  QueryDataFromRequest(query_data): QueryDataFromRequest,
  Path(user_con_profile_id): Path<i64>,
) -> Result<impl IntoResponse, StatusCode> {
  // TODO authorization

  let subject_profile = user_con_profiles::Entity::find_by_id(user_con_profile_id)
    .one(query_data.db())
    .await
    .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?;

  let Some(subject_profile)= subject_profile else {
    return Err(StatusCode::NOT_FOUND);
  };

  let signups_and_runs = subject_profile
    .find_related(signups::Entity)
    .find_also_related(runs::Entity)
    .filter(signups::Column::State.ne("withdrawn"))
    .all(query_data.db())
    .await
    .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?;

  let mut runs_by_id = signups_and_runs
    .iter()
    .filter_map(|(_signup, run)| run.as_ref().map(|run| (run.id, run.clone())))
    .collect::<HashMap<_, _>>();

  let team_member_events = events::Entity::find()
    .inner_join(team_members::Entity)
    .filter(team_members::Column::UserConProfileId.eq(subject_profile.id))
    .all(query_data.db())
    .await
    .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?;

  let signed_up_event_ids = runs_by_id
    .values()
    .map(|run| run.event_id)
    .collect::<HashSet<_>>();
  let additional_events = team_member_events
    .iter()
    .filter(|event| !signed_up_event_ids.contains(&event.id))
    .cloned()
    .collect::<Vec<_>>();

  if !additional_events.is_empty() {
    runs_by_id.extend(
      runs::Entity::find()
        .filter(runs::Column::EventId.is_in(additional_events.iter().map(|event| event.id)))
        .all(query_data.db())
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|run| (run.id, run)),
    )
  }

  let runs_by_event_id = runs_by_id
    .values()
    .map(|run| (run.event_id, run.clone()))
    .into_group_map();

  let events_by_id = events::Entity::find()
    .filter(
      events::Column::Id.is_in(
        signed_up_event_ids
          .iter()
          .chain(additional_events.iter().map(|event| &event.id))
          .copied(),
      ),
    )
    .all(query_data.db())
    .await
    .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .chain(team_member_events.iter().cloned())
    .map(|event| (event.id, event))
    .collect::<HashMap<_, _>>();

  let rooms_by_run_id = rooms_runs::Entity::find()
    .filter(rooms_runs::Column::RunId.is_in(runs_by_id.keys().copied()))
    .find_also_related(rooms::Entity)
    .all(query_data.db())
    .await
    .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .fold(HashMap::new(), |mut acc, (room_run, room)| {
      let rooms = acc.entry(room_run.run_id).or_insert(vec![]);
      if let Some(room) = room {
        rooms.push(room);
      }
      acc
    });

  let mut active_signups = signups_and_runs
    .iter()
    .map(|(signup, _run)| signup.clone())
    .collect::<Vec<_>>();
  active_signups.sort_by_key(|signup| runs_by_id.get(&signup.run_id).and_then(|run| run.starts_at));

  let template = SingleUserPrintableTemplate {
    active_signups,
    application_styles_js_url: url_with_possible_host(
      "/packs/application-styles.js",
      env::var("ASSETS_HOST").ok(),
    ),
    convention_name: query_data
      .convention()
      .and_then(|c| c.name.as_deref())
      .unwrap_or_default()
      .to_string(),
    events_by_id,
    page_title: "".to_string(),
    report_type: "Single user printable".to_string(),
    user_con_profile: subject_profile,
    runs_by_id,
    rooms_by_run_id,
    team_member_events,
    runs_by_event_id,
  };

  template
    .render()
    .map(Html)
    .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)
}
