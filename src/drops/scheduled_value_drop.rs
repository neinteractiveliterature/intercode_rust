use std::{
  fmt::Debug,
  sync::atomic::{AtomicI64, Ordering},
};

use chrono::{TimeZone, Utc};
use intercode_timespan::{ScheduledValue, TimespanWithValue};
use liquid::model::DateTime;
use seawater::liquid_drop_impl;
use serde::Serialize;

use super::{utils::date_time_to_liquid_date_time, DropContext, TimespanWithValueDrop};

#[derive(Clone, Debug)]
pub struct ScheduledValueDrop<
  Tz: TimeZone + Debug,
  V: Serialize + Debug + Clone + Default + Send + Sync,
> where
  Tz::Offset: Send + Sync,
{
  scheduled_value: ScheduledValue<Tz, V>,
  context: DropContext,
  id: i64,
}

static NEXT_ID: AtomicI64 = AtomicI64::new(0);

#[liquid_drop_impl(i64, DropContext)]
impl<
    Tz: TimeZone + Debug + Eq + Send + Sync,
    V: Serialize + Debug + Clone + Default + Send + Sync,
  > ScheduledValueDrop<Tz, V>
where
  Tz::Offset: Send + Sync,
{
  pub fn new(scheduled_value: ScheduledValue<Tz, V>, context: DropContext) -> Self {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    Self {
      scheduled_value,
      context,
      id,
    }
  }

  pub fn now(&self) -> DateTime {
    date_time_to_liquid_date_time(&Utc::now()).unwrap()
  }

  pub fn next_value_change(&self) -> Option<DateTime> {
    self
      .scheduled_value
      .next_value_change_after(Utc::now())
      .and_then(|t| date_time_to_liquid_date_time(t))
  }

  pub fn covers_all_time(&self) -> bool {
    // TODO think about if we need to handle this differently
    true
  }

  pub fn timespans(&self) -> Vec<TimespanWithValueDrop<Tz, Tz, liquid::model::Value>> {
    let filtered_timespans: Vec<TimespanWithValueDrop<Tz, Tz, V>> = self
      .scheduled_value
      .into_iter()
      .map(|twv| TimespanWithValueDrop::new(twv, self.context.clone()))
      .filter_map(|twv_drop| twv_drop.into())
      .collect();

    filtered_timespans
      .iter()
      .map(|twv_drop| {
        TimespanWithValueDrop::new(
          TimespanWithValue {
            timespan: twv_drop.timespan_with_value.timespan.clone(),
            value: liquid::model::to_value(&twv_drop.timespan_with_value.value)
              .unwrap_or(liquid::model::Value::Nil),
          },
          self.context.clone(),
        )
      })
      .collect()
  }

  pub fn current_value(&self) -> Option<liquid::model::Value> {
    self
      .scheduled_value
      .value_at(Utc::now())
      .map(|value| liquid::model::to_value(&value).unwrap_or(liquid::model::Value::Nil))
  }

  pub fn current_value_change(&self) -> Option<DateTime> {
    self
      .scheduled_value
      .current_value_changed_at(Utc::now())
      .and_then(|t| date_time_to_liquid_date_time(t))
  }

  pub fn next_value(&self) -> Option<liquid::model::Value> {
    self
      .scheduled_value
      .next_value_change_after(Utc::now())
      .and_then(|t| self.scheduled_value.value_at(t.to_owned()))
      .map(|value| liquid::model::to_value(&value).unwrap_or(liquid::model::Value::Nil))
  }
}
