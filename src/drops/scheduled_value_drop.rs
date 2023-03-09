use std::fmt::Debug;

use chrono::{TimeZone, Utc};
use intercode_timespan::{ScheduledValue, TimespanWithValue};
use liquid::model::DateTime;
use seawater::{liquid_drop_impl, liquid_drop_struct};
use serde::Serialize;

use super::{utils::date_time_to_liquid_date_time, DropContext, TimespanWithValueDrop};

#[liquid_drop_struct]
pub struct ScheduledValueDrop<Tz: TimeZone + Debug, V: Serialize + Debug + Clone + Default> {
  scheduled_value: ScheduledValue<Tz, V>,
  context: DropContext,
}

#[liquid_drop_impl]
impl<Tz: TimeZone + Debug, V: Serialize + Debug + Clone + Default> ScheduledValueDrop<Tz, V> {
  pub fn new(scheduled_value: ScheduledValue<Tz, V>, context: DropContext) -> Self {
    Self {
      scheduled_value,
      context,
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
