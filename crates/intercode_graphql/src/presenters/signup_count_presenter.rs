use std::collections::HashMap;

use intercode_entities::signups;
use sea_orm::{
  sea_query::Expr, ColumnTrait, DbErr, EntityTrait, FromQueryResult, QueryFilter, QuerySelect,
};
use seawater::ConnectionWrapper;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SignupCountDataCountedStatus {
  Counted,
  NotCounted,
}

impl From<bool> for SignupCountDataCountedStatus {
  fn from(counted: bool) -> Self {
    match counted {
      true => Self::Counted,
      false => Self::NotCounted,
    }
  }
}

fn effective_bucket_key<'a>(
  state: &'a str,
  bucket_key: Option<&'a str>,
  requested_bucket_key: Option<&'a str>,
) -> Option<&'a str> {
  match state {
    "waitlisted" => requested_bucket_key,
    _ => bucket_key,
  }
}

#[derive(FromQueryResult)]
struct SignupCountDataRow {
  run_id: i64,
  state: String,
  bucket_key: Option<String>,
  requested_bucket_key: Option<String>,
  counted: bool,
  count: u64,
}

#[derive(Default, Clone, Debug)]
pub struct RunSignupCounts {
  pub count_by_state_and_bucket_key_and_counted:
    HashMap<String, HashMap<Option<String>, HashMap<SignupCountDataCountedStatus, u64>>>,
}

impl RunSignupCounts {
  pub fn add(&mut self, state: &str, bucket_key: Option<&str>, counted: bool, count: u64) {
    let state_entry = self
      .count_by_state_and_bucket_key_and_counted
      .entry(state.to_string());
    let count_by_bucket_key = state_entry.or_insert_with(Default::default);
    let count_by_counted = count_by_bucket_key
      .entry(bucket_key.map(|key| key.to_string()))
      .or_insert_with(Default::default);
    let current_count_entry = count_by_counted.entry(counted.into());
    current_count_entry
      .and_modify(|current_count| *current_count += count)
      .or_insert(count);
  }

  pub fn counted_signups_by_state(&self, state: &str) -> u64 {
    let count_by_bucket_key_and_counted = self.count_by_state_and_bucket_key_and_counted.get(state);

    if let Some(count_by_bucket_key_and_counted) = count_by_bucket_key_and_counted {
      count_by_bucket_key_and_counted
        .values()
        .fold(0, |acc, count_by_counted| {
          acc
            + count_by_counted
              .get(&SignupCountDataCountedStatus::Counted)
              .unwrap_or(&0)
        })
    } else {
      0
    }
  }

  pub fn not_counted_signups_by_state(&self, state: &str) -> u64 {
    let count_by_bucket_key_and_counted = self.count_by_state_and_bucket_key_and_counted.get(state);

    if let Some(count_by_bucket_key_and_counted) = count_by_bucket_key_and_counted {
      count_by_bucket_key_and_counted
        .values()
        .fold(0, |acc, count_by_counted| {
          acc
            + count_by_counted
              .get(&SignupCountDataCountedStatus::NotCounted)
              .unwrap_or(&0)
        })
    } else {
      0
    }
  }
}

pub async fn load_signup_count_data_for_run_ids<I: IntoIterator<Item = i64>>(
  db: &ConnectionWrapper,
  run_ids: I,
) -> Result<HashMap<i64, RunSignupCounts>, DbErr> {
  let count_data = signups::Entity::find()
    .filter(signups::Column::RunId.is_in(run_ids))
    .select_only()
    .column(signups::Column::RunId)
    .column(signups::Column::State)
    .column(signups::Column::BucketKey)
    .column(signups::Column::RequestedBucketKey)
    .column(signups::Column::Counted)
    .column_as(Expr::col(signups::Column::Id).count(), "count")
    .group_by(signups::Column::RunId)
    .group_by(signups::Column::State)
    .group_by(signups::Column::BucketKey)
    .group_by(signups::Column::RequestedBucketKey)
    .group_by(signups::Column::Counted)
    .into_model::<SignupCountDataRow>()
    .all(db)
    .await?;

  Ok(count_data.iter().fold(Default::default(), |mut acc, row| {
    let entry = acc.entry(row.run_id);
    let run_counts = entry.or_insert_with(Default::default);

    let effective_bucket_key = effective_bucket_key(
      &row.state,
      row.bucket_key.as_deref(),
      row.requested_bucket_key.as_deref(),
    );
    run_counts.add(&row.state, effective_bucket_key, row.counted, row.count);

    acc
  }))
}
