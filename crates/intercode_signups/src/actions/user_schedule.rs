use std::{collections::HashMap, io::BufWriter};

use axum::{body::Bytes, extract::Path};
use chrono::{format::StrftimeItems, Duration, NaiveDateTime};
use chrono_tz::UTC;
use http::{header::CONTENT_TYPE, StatusCode};
use ics::{properties, ICalendar};
use intercode_entities::{events, runs, signups, user_con_profiles};
use intercode_graphql_loaders::LoaderManager;
use intercode_server::QueryDataFromRequest;
use itertools::Itertools;
use once_cell::sync::Lazy;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use seawater::loaders::{ExpectModel, ExpectModels};
use tracing::log::error;
use uuid::Uuid;

static ICAL_DATE_FORMAT: Lazy<Vec<chrono::format::Item<'static>>> =
  Lazy::new(|| StrftimeItems::new("%Y%m%dT%H%M%SZ").collect());

fn naive_date_time_to_ical_date(datetime: NaiveDateTime) -> String {
  datetime
    .and_local_timezone(UTC)
    .unwrap()
    .format_with_items(ICAL_DATE_FORMAT.iter())
    .to_string()
}

fn event_summary(event: &events::Model, run: &runs::Model, signup: &signups::Model) -> String {
  let mut title = event.title.to_string();
  if signup.state == "waitlisted" {
    title = format!("[WAITLISTED] {}", title);
  }
  if let Some(title_suffix) = run.title_suffix.as_deref() {
    title = format!("{} ({})", title, title_suffix);
  }

  title
}

pub async fn user_schedule(
  QueryDataFromRequest(query_data): QueryDataFromRequest,
  Path(ical_secret): Path<String>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
  let Some(convention) = query_data.convention() else {
    return Err(StatusCode::NOT_FOUND);
  };

  let user_con_profile = user_con_profiles::Entity::find()
    .filter(user_con_profiles::Column::ConventionId.eq(convention.id))
    .filter(user_con_profiles::Column::IcalSecret.eq(ical_secret))
    .one(query_data.db())
    .await
    .map_err(|err| {
      error!("{}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let Some(user_con_profile) = user_con_profile else {
    return Err(StatusCode::NOT_FOUND);
  };

  let mut cal = ICalendar::new(
    "2.0",
    "-//New England Interactive Literature//Intercode//EN",
  );
  cal.push(properties::Name::new(format!(
    "{} Schedule for {}",
    convention.name.as_deref().unwrap_or_default(),
    user_con_profile.name()
  )));

  // TODO VTIMEZONE?
  // let timezone = convention
  //   .timezone_name
  //   .as_deref()
  //   .and_then(|tz_name| chrono_tz::Tz::from_str(tz_name).ok())
  //   .unwrap_or(chrono_tz::Tz::UTC);

  // let timezone_component = Component::new("VTIMEZONE");
  // timezone.

  let loaders = LoaderManager::new(query_data.db().clone());

  let signups_loader_result = loaders
    .user_con_profile_signups()
    .load_one(user_con_profile.id)
    .await
    .map_err(|err| {
      error!("{}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;
  let signups = signups_loader_result
    .expect_models()
    .map_err(|err| {
      error!("{}", err.message);
      StatusCode::INTERNAL_SERVER_ERROR
    })?
    .iter()
    .filter(|signup| signup.state != "withdrawn")
    .collect::<Vec<_>>();

  let runs_by_id_result = loaders
    .runs_by_id()
    .load_many(signups.iter().map(|signup| signup.run_id))
    .await
    .map_err(|err| {
      error!("{}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;
  let runs_by_id = runs_by_id_result
    .iter()
    .filter_map(|(run_id, result)| result.expect_one().map(|run| (*run_id, run)).ok())
    .collect::<HashMap<_, _>>();

  let run_rooms_result = loaders
    .run_rooms()
    .load_many(runs_by_id.keys().copied())
    .await
    .map_err(|err| {
      error!("{}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;
  let rooms_by_run_id = run_rooms_result
    .iter()
    .filter_map(|(run_id, rooms_result)| {
      rooms_result
        .expect_models()
        .map(|rooms| (*run_id, rooms))
        .ok()
    })
    .collect::<HashMap<_, _>>();

  let events_by_id_result = loaders
    .events_by_id()
    .load_many(runs_by_id.values().map(|run| run.event_id))
    .await
    .map_err(|err| {
      error!("{}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;
  let events_by_id = events_by_id_result
    .iter()
    .filter_map(|(event_id, result)| result.expect_one().map(|event| (*event_id, event)).ok())
    .collect::<HashMap<_, _>>();

  for signup in signups {
    let Some(run) = runs_by_id.get(&signup.run_id) else {
      continue;
    };
    let Some(event) = events_by_id.get(&run.event_id) else {
      continue;
    };
    let mut rooms = rooms_by_run_id
      .get(&signup.run_id)
      .map(|rooms| (*rooms).to_owned())
      .unwrap_or_default();
    rooms.sort_by_cached_key(|room| room.name.to_owned().unwrap_or_default());

    let mut ical_event = ics::Event::new(
      Uuid::new_v4().to_string(),
      naive_date_time_to_ical_date(signup.created_at),
    );
    ical_event.push(properties::DtStart::new(naive_date_time_to_ical_date(
      run.starts_at,
    )));
    ical_event.push(properties::DtEnd::new(naive_date_time_to_ical_date(
      run.starts_at + Duration::seconds(event.length_seconds.into()),
    )));
    ical_event.push(properties::Summary::new(event_summary(event, run, signup)));
    ical_event.push(properties::Location::new(
      rooms
        .iter()
        .filter_map(|room| room.name.as_deref())
        .join(", "),
    ));
    if let Some(short_blurb) = &event.short_blurb {
      ical_event.push(properties::Description::new(short_blurb));
    }
    ical_event.push(properties::URL::new(format!(
      "https://{}/events/{}",
      convention.domain, event.id
    )));

    cal.add_event(ical_event);
  }

  let mut bytes: Vec<u8> = Vec::with_capacity(1024);
  let buf = BufWriter::new(&mut bytes);
  cal.write(buf).map_err(|err| {
    error!("{}", err);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let body = Bytes::from(bytes);

  Ok(([(CONTENT_TYPE, "text/calendar")], body))
}
