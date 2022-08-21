use std::{fmt::Debug, sync::Arc};

use chrono::TimeZone;
use i18n_embed::fluent::FluentLanguageLoader;
use intercode_timespan::TimespanWithValue;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use liquid::model::DateTime;
use serde::Serialize;

use super::utils::date_time_to_liquid_date_time;

#[liquid_drop_struct]
pub struct TimespanWithValueDrop<
  StartTz: TimeZone + Debug,
  FinishTz: TimeZone + Debug,
  V: Clone + Serialize + Debug,
> {
  pub timespan_with_value: TimespanWithValue<StartTz, FinishTz, V>,
  language_loader: Arc<FluentLanguageLoader>,
}

#[liquid_drop_impl]
impl<StartTz: TimeZone + Debug, FinishTz: TimeZone + Debug, V: Clone + Serialize + Debug>
  TimespanWithValueDrop<StartTz, FinishTz, V>
{
  pub fn new(
    timespan_with_value: TimespanWithValue<StartTz, FinishTz, V>,
    language_loader: Arc<FluentLanguageLoader>,
  ) -> Self {
    TimespanWithValueDrop {
      timespan_with_value,
      language_loader,
    }
  }

  pub fn start(&self) -> Option<DateTime> {
    self
      .timespan_with_value
      .timespan
      .start
      .as_ref()
      .and_then(date_time_to_liquid_date_time)
  }

  pub fn finish(&self) -> Option<DateTime> {
    self
      .timespan_with_value
      .timespan
      .finish
      .as_ref()
      .and_then(date_time_to_liquid_date_time)
  }

  pub fn description(&self) -> String {
    // TODO
    "TODO".to_string()
  }

  pub fn description_without_value(&self) -> String {
    // TODO
    "TODO".to_string()
  }

  pub fn short_description(&self) -> String {
    // TODO
    "TODO".to_string()
  }

  pub fn short_description_without_value(&self) -> String {
    // TODO
    "TODO".to_string()
  }

  pub fn value(&self) -> liquid::model::Value {
    liquid::model::to_value(&self.timespan_with_value.value).unwrap_or(liquid::model::Value::Nil)
  }
}

impl<StartTz: TimeZone + Debug, FinishTz: TimeZone + Debug, V: Clone + Serialize + Debug>
  From<TimespanWithValueDrop<StartTz, FinishTz, Option<V>>>
  for Option<TimespanWithValueDrop<StartTz, FinishTz, V>>
{
  fn from(drop: TimespanWithValueDrop<StartTz, FinishTz, Option<V>>) -> Self {
    match drop.timespan_with_value.value {
      Some(value) => Some(TimespanWithValueDrop::new(
        TimespanWithValue {
          value,
          timespan: drop.timespan_with_value.timespan,
        },
        drop.language_loader.clone(),
      )),
      None => None,
    }
  }
}
