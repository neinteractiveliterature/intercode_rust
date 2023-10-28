use std::collections::{HashMap, HashSet};

use askama::Template;
use futures::try_join;
use intercode_entities::{
  events, rooms, rooms_runs, runs, signups, team_members, user_con_profiles, UserNames,
};
use intercode_graphql_core::query_data::QueryData;
use intercode_inflector::IntercodeInflector;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, ModelTrait, QueryFilter};

use crate::actions::filters;

use super::event_run_helpers::EventRunHelpers;

#[derive(Template)]
#[template(path = "reports/per_user_single.html.j2")]
pub struct PerUserSingleTemplate {
  inflector: IntercodeInflector,
  user_con_profile: user_con_profiles::Model,
  active_signups: Vec<signups::Model>,
  pub runs_by_id: HashMap<i64, runs::Model>,
  pub rooms_by_run_id: HashMap<i64, Vec<rooms::Model>>,
  pub events_by_id: HashMap<i64, events::Model>,
  pub team_member_events: Vec<events::Model>,
}

impl EventRunHelpers for PerUserSingleTemplate {
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

impl PerUserSingleTemplate {
  pub async fn load(
    query_data: &QueryData,
    subject_profile: user_con_profiles::Model,
  ) -> Result<Self, DbErr> {
    let signups_and_runs = subject_profile
      .find_related(signups::Entity)
      .find_also_related(runs::Entity)
      .filter(signups::Column::State.ne("withdrawn"))
      .all(query_data.db())
      .await?;

    let mut runs_by_id = signups_and_runs
      .iter()
      .filter_map(|(_signup, run)| run.as_ref().map(|run| (run.id, run.clone())))
      .collect::<HashMap<_, _>>();

    let team_member_events = events::Entity::find()
      .inner_join(team_members::Entity)
      .filter(team_members::Column::UserConProfileId.eq(subject_profile.id))
      .all(query_data.db())
      .await?;

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
          .await?
          .into_iter()
          .map(|run| (run.id, run)),
      )
    }

    let all_run_ids = runs_by_id.keys().copied().collect::<HashSet<_>>();

    let (events_by_id, rooms_by_run_id) = try_join!(
      async {
        Ok::<_, DbErr>(
          events::Entity::find()
            .filter(events::Column::Id.is_in(all_run_ids.clone()))
            .all(query_data.db())
            .await?
            .into_iter()
            .chain(team_member_events.iter().cloned())
            .map(|event| (event.id, event))
            .collect::<HashMap<_, _>>(),
        )
      },
      async {
        Ok::<_, DbErr>(
          rooms_runs::Entity::find()
            .filter(rooms_runs::Column::RunId.is_in(all_run_ids.clone()))
            .find_also_related(rooms::Entity)
            .all(query_data.db())
            .await?
            .into_iter()
            .fold(HashMap::new(), |mut acc, (room_run, room)| {
              let rooms: &mut Vec<rooms::Model> = acc.entry(room_run.run_id).or_default();
              if let Some(room) = room {
                rooms.push(room);
              }
              acc
            }),
        )
      }
    )?;

    let mut active_signups = signups_and_runs
      .iter()
      .map(|(signup, _run)| signup.clone())
      .collect::<Vec<_>>();
    active_signups
      .sort_by_cached_key(|signup| runs_by_id.get(&signup.run_id).and_then(|run| run.starts_at));

    Ok(Self {
      inflector: IntercodeInflector::new(),
      user_con_profile: subject_profile,
      active_signups,
      runs_by_id,
      rooms_by_run_id,
      events_by_id,
      team_member_events,
    })
  }

  fn titleized_state(&self, state: &str) -> String {
    self.inflector.titleize(state)
  }
}
