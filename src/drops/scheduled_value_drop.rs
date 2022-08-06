use chrono::{TimeZone, Utc};
use i18n_embed::fluent::FluentLanguageLoader;
use intercode_timespan::ScheduledValue;
use lazy_liquid_value_view::DropResult;
use liquid::model::DateTime;
use serde::{Deserialize, Serialize};

use super::{utils::date_time_to_liquid_date_time, TimespanWithValueDrop};

#[derive(Serialize, Deserialize, Debug)]
pub struct ScheduledValueDrop<V: Clone + Default> {
  now: DateTime,
  covers_all_time: bool,
  timespans: Vec<TimespanWithValueDrop<V>>,
  current_value: Option<V>,
  current_value_change: Option<DateTime>,
  next_value: Option<V>,
  next_value_change: Option<DateTime>,
}

impl<V: Clone + Default + std::fmt::Debug> ScheduledValueDrop<V> {
  pub fn new<Tz: TimeZone + std::fmt::Debug>(
    scheduled_value: ScheduledValue<Tz, V>,
    language_loader: &FluentLanguageLoader,
  ) -> Self {
    let now = Utc::now();
    let next_value_change = scheduled_value
      .next_value_change_after(now)
      .map(|t| t.to_owned());

    ScheduledValueDrop {
      now: date_time_to_liquid_date_time(now).unwrap(),
      covers_all_time: true, // TODO think about if we need to handle this differently
      timespans: scheduled_value
        .into_iter()
        .map(|twv| TimespanWithValueDrop::new(twv, language_loader))
        .filter_map(|twv_drop| twv_drop.into())
        .collect(),
      current_value: scheduled_value.value_at(now),
      current_value_change: scheduled_value
        .current_value_changed_at(now)
        .and_then(|t| date_time_to_liquid_date_time(t.to_owned())),
      next_value: next_value_change
        .as_ref()
        .and_then(|t| scheduled_value.value_at(t.to_owned())),
      next_value_change: next_value_change.and_then(date_time_to_liquid_date_time),
    }
  }
}

impl<'a, V: Clone + Default + Serialize> From<ScheduledValueDrop<V>> for DropResult<'a> {
  fn from(drop: ScheduledValueDrop<V>) -> Self {
    DropResult::new(liquid::model::to_value(&drop).unwrap())
  }
}
