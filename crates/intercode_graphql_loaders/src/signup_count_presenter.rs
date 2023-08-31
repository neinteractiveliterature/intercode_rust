use std::collections::HashMap;

use futures::try_join;
use intercode_entities::{events, runs, signups, RegistrationPolicy};
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
  counted: Option<bool>,
  count: i64,
}

#[derive(Default, Clone, Debug)]
pub struct RunSignupCounts {
  pub registration_policy: RegistrationPolicy,
  pub count_by_state_and_bucket_key_and_counted:
    HashMap<String, HashMap<String, HashMap<SignupCountDataCountedStatus, i64>>>,
}

impl RunSignupCounts {
  pub fn add(&mut self, state: &str, bucket_key: Option<&str>, counted: bool, count: i64) {
    let state_entry = self
      .count_by_state_and_bucket_key_and_counted
      .entry(state.to_string());
    let count_by_bucket_key = state_entry.or_insert_with(Default::default);
    let count_by_counted = count_by_bucket_key
      .entry(bucket_key.map(|key| key.to_string()).unwrap_or_default())
      .or_insert_with(Default::default);
    let current_count_entry = count_by_counted.entry(counted.into());
    current_count_entry
      .and_modify(|current_count| *current_count += count)
      .or_insert(count);
  }

  pub fn signup_count(
    &self,
    state: &str,
    bucket_key: &str,
    counted: SignupCountDataCountedStatus,
  ) -> i64 {
    let Some(count_by_bucket_key_and_counted) =
      self.count_by_state_and_bucket_key_and_counted.get(state)
    else {
      return 0;
    };

    let Some(count_by_counted) = count_by_bucket_key_and_counted.get(bucket_key) else {
      return 0;
    };

    count_by_counted.get(&counted).copied().unwrap_or(0)
  }

  pub fn counted_signups_by_state(&self, state: &str) -> i64 {
    self
      .registration_policy
      .all_buckets()
      .fold(0, |acc, bucket| {
        acc + self.signup_count(state, &bucket.key, SignupCountDataCountedStatus::Counted)
      })
  }

  pub fn not_counted_signups_by_state(&self, state: &str) -> i64 {
    self
      .registration_policy
      .all_buckets()
      .fold(0, |acc, bucket| {
        acc + self.signup_count(state, &bucket.key, SignupCountDataCountedStatus::NotCounted)
      })
  }

  pub fn counted_key_for_state(&self, state: &str) -> SignupCountDataCountedStatus {
    if state != "confirmed" || self.registration_policy.only_uncounted() {
      SignupCountDataCountedStatus::NotCounted
    } else {
      SignupCountDataCountedStatus::Counted
    }
  }

  pub fn bucket_description_for_state(&self, bucket_key: &str, state: &str) -> String {
    let Some(bucket) = self.registration_policy.bucket_with_key(bucket_key) else {
      return "".to_string();
    };

    let counted = if bucket.is_not_counted() {
      SignupCountDataCountedStatus::NotCounted
    } else {
      self.counted_key_for_state(state)
    };

    let count = self.signup_count(state, bucket_key, counted);
    if self.registration_policy.all_buckets().count() == 1 {
      count.to_string()
    } else {
      format!("{}: {}", bucket.name.trim(), count)
    }
  }

  pub fn all_bucket_descriptions_for_state(&self, state: &str) -> String {
    let buckets = self.registration_policy.all_buckets().collect::<Vec<_>>();
    let mut bucket_texts = Vec::<String>::with_capacity(buckets.len() + 1);

    for bucket in buckets {
      bucket_texts.push(self.bucket_description_for_state(&bucket.key, state));
    }

    let no_preference_waitlist_count =
      self.signup_count("waitlisted", "", SignupCountDataCountedStatus::NotCounted);
    if state == "waitlisted" && no_preference_waitlist_count > 0 {
      bucket_texts.push(format!("No preference: {}", no_preference_waitlist_count));
    }

    bucket_texts.join(", ")
  }
}

pub async fn load_signup_count_data_for_run_ids<I: Iterator<Item = i64>>(
  db: &ConnectionWrapper,
  run_ids: I,
) -> Result<HashMap<i64, RunSignupCounts>, DbErr> {
  let run_ids = run_ids.collect::<Vec<_>>();

  let (registration_policy_by_run_id, count_data) = try_join!(
    async {
      Ok(
        runs::Entity::find()
          .filter(runs::Column::Id.is_in(run_ids.clone()))
          .find_also_related(events::Entity)
          .all(db)
          .await?
          .into_iter()
          .filter_map(|(run, event)| {
            event.and_then(|event| {
              event.registration_policy.map(|json| {
                (
                  run.id,
                  serde_json::from_value::<RegistrationPolicy>(json).unwrap(),
                )
              })
            })
          })
          .collect::<HashMap<_, _>>(),
      )
    },
    signups::Entity::find()
      .filter(signups::Column::RunId.is_in(run_ids.clone()))
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
  )?;

  Ok(count_data.iter().fold(Default::default(), |mut acc, row| {
    let entry = acc.entry(row.run_id);
    let run_counts = entry.or_insert_with(|| RunSignupCounts {
      registration_policy: registration_policy_by_run_id
        .get(&row.run_id)
        .cloned()
        .unwrap_or_default(),
      count_by_state_and_bucket_key_and_counted: Default::default(),
    });

    let effective_bucket_key = effective_bucket_key(
      &row.state,
      row.bucket_key.as_deref(),
      row.requested_bucket_key.as_deref(),
    );
    run_counts.add(
      &row.state,
      effective_bucket_key,
      row.counted.unwrap_or(false),
      row.count,
    );

    acc
  }))
}
