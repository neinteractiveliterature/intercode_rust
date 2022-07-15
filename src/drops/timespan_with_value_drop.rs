use chrono::TimeZone;
use i18n_embed::fluent::FluentLanguageLoader;
use intercode_timespan::TimespanWithValue;
use liquid::model::DateTime;
use serde::{Deserialize, Serialize};

use super::utils::date_time_to_liquid_date_time;

#[derive(Serialize, Deserialize, Debug)]
pub struct TimespanWithValueDrop<V: Clone + Default> {
  pub start: Option<DateTime>,
  pub finish: Option<DateTime>,
  pub description: String,
  pub description_without_value: String,
  pub short_description: String,
  pub short_description_without_value: String,
  pub value: V,
}

impl<V: Clone + Default> TimespanWithValueDrop<V> {
  pub fn new<StartTz: TimeZone, FinishTz: TimeZone>(
    twv: TimespanWithValue<StartTz, FinishTz, V>,
    _language_loader: &FluentLanguageLoader,
  ) -> Self {
    TimespanWithValueDrop {
      start: twv.timespan.start.and_then(date_time_to_liquid_date_time),
      finish: twv.timespan.finish.and_then(date_time_to_liquid_date_time),
      description: "TODO".to_string(),
      description_without_value: "TODO".to_string(),
      short_description: "TODO".to_string(),
      short_description_without_value: "TODO".to_string(),
      value: twv.value,
    }
  }
}

impl<V: Clone + Default> From<TimespanWithValueDrop<Option<V>>>
  for Option<TimespanWithValueDrop<V>>
{
  fn from(drop: TimespanWithValueDrop<Option<V>>) -> Self {
    match drop.value {
      Some(value) => Some(TimespanWithValueDrop {
        start: drop.start,
        finish: drop.finish,
        description: drop.description,
        description_without_value: drop.description_without_value,
        short_description: drop.short_description,
        short_description_without_value: drop.short_description_without_value,
        value,
      }),
      None => None,
    }
  }
}
