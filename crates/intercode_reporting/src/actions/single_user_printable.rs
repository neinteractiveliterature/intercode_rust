use std::{
  collections::{HashMap, HashSet},
  env,
};

use askama::Template;
use axum::{
  debug_handler,
  extract::Path,
  response::{Html, IntoResponse},
};
use http::StatusCode;
use intercode_entities::{
  event_categories, events, runs, signups, team_members, user_con_profiles,
};
use intercode_graphql_core::rendering_utils::url_with_possible_host;
use intercode_graphql_loaders::signup_count_presenter::{
  load_signup_count_data_for_run_ids, RunSignupCounts,
};
use intercode_server::QueryDataFromRequest;
use itertools::Itertools;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tracing::log::error;

use super::{per_event_single::PerEventSingleTemplate, per_user_single::PerUserSingleTemplate};

#[derive(Template)]
#[template(path = "reports/single_user_printable.html.j2")]
struct SingleUserPrintableTemplate {
  page_title: String,
  application_styles_js_url: String,
  convention_name: String,
  report_type: String,
  per_user_single: PerUserSingleTemplate,
  runs_by_event_id: HashMap<i64, Vec<runs::Model>>,
  team_member_name_by_event_category_id: HashMap<i64, String>,
  team_member_profiles_by_event_id: HashMap<i64, Vec<user_con_profiles::Model>>,
  run_signup_counts_by_run_id: HashMap<i64, RunSignupCounts>,
  signups_by_run_id: HashMap<i64, Vec<signups::Model>>,
  user_con_profiles_by_id: HashMap<i64, user_con_profiles::Model>,
}

impl SingleUserPrintableTemplate {
  fn render_per_user_single(&self) -> String {
    self.per_user_single.render().unwrap()
  }

  fn render_per_event_single(&self, event: &events::Model) -> String {
    let per_event_single = PerEventSingleTemplate::new(
      self.per_user_single.runs_by_id.clone(),
      self.per_user_single.rooms_by_run_id.clone(),
      self.per_user_single.events_by_id.clone(),
      self.runs_by_event_id.clone(),
      event.clone(),
      self.team_member_name_by_event_category_id.clone(),
      self.team_member_profiles_by_event_id.clone(),
      self.run_signup_counts_by_run_id.clone(),
      self.signups_by_run_id.clone(),
      self.user_con_profiles_by_id.clone(),
    );

    per_event_single.render().unwrap()
  }
}

#[debug_handler]
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

  let per_user_single = PerUserSingleTemplate::load(&query_data, subject_profile)
    .await
    .map_err(|err| {
      error!("{}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let runs_by_event_id = per_user_single
    .runs_by_id
    .values()
    .map(|run| (run.event_id, run.clone()))
    .into_group_map();

  let team_member_name_by_event_category_id = event_categories::Entity::find()
    .filter(
      event_categories::Column::Id.is_in(
        per_user_single
          .events_by_id
          .values()
          .map(|event| event.event_category_id),
      ),
    )
    .all(query_data.db())
    .await
    .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|category| (category.id, category.team_member_name))
    .collect::<HashMap<_, _>>();

  let team_member_profiles_by_event_id = team_members::Entity::find()
    .filter(team_members::Column::EventId.is_in(per_user_single.events_by_id.keys().copied()))
    .find_also_related(user_con_profiles::Entity)
    .all(query_data.db())
    .await
    .map_err(|err| {
      error!("{}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?
    .into_iter()
    .filter_map(|(team_member, profile)| profile.map(|p| (team_member.event_id.unwrap(), p)))
    .into_group_map();

  let run_signup_counts_by_run_id =
    load_signup_count_data_for_run_ids(query_data.db(), per_user_single.runs_by_id.keys().copied())
      .await
      .map_err(|err| {
        error!("{}", err);
        StatusCode::INTERNAL_SERVER_ERROR
      })?;

  let signups_by_run_id = signups::Entity::find()
    .filter(signups::Column::RunId.is_in(per_user_single.runs_by_id.keys().copied()))
    .all(query_data.db())
    .await
    .map_err(|err| {
      error!("{}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?
    .into_iter()
    .map(|signup| (signup.run_id, signup))
    .into_group_map();

  let user_con_profile_ids = signups_by_run_id
    .values()
    .flat_map(|signups| {
      signups
        .iter()
        .map(|signup| &signup.user_con_profile_id)
        .copied()
    })
    .collect::<HashSet<_>>();

  let user_con_profiles_by_id = user_con_profiles::Entity::find()
    .filter(user_con_profiles::Column::Id.is_in(user_con_profile_ids))
    .all(query_data.db())
    .await
    .map_err(|err| {
      error!("{}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?
    .into_iter()
    .map(|user_con_profile| (user_con_profile.id, user_con_profile))
    .collect::<HashMap<_, _>>();

  let template = SingleUserPrintableTemplate {
    application_styles_js_url: url_with_possible_host(
      "/packs/application-styles.js",
      env::var("ASSETS_HOST").ok(),
    ),
    convention_name: query_data
      .convention()
      .and_then(|c| c.name.as_deref())
      .unwrap_or_default()
      .to_string(),
    page_title: "".to_string(),
    report_type: "Single user printable".to_string(),
    per_user_single,
    runs_by_event_id,
    team_member_name_by_event_category_id,
    team_member_profiles_by_event_id,
    run_signup_counts_by_run_id,
    signups_by_run_id,
    user_con_profiles_by_id,
  };

  template
    .render()
    .map(Html)
    .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)
}
