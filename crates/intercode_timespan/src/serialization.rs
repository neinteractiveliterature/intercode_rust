use std::fmt::Display;

use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::{ScheduledValue, Timespan, TimespanWithValue};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SerializedTimespan {
  start: Option<String>,
  finish: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SerializedTimespanWithValue<V> {
  start: Option<String>,
  finish: Option<String>,
  value: V,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SerializedScheduledValue<V> {
  timespans: Vec<SerializedTimespanWithValue<V>>,
}

impl<StartTz: TimeZone, FinishTz: TimeZone> From<SerializedTimespan> for Timespan<StartTz, FinishTz>
where
  DateTime<StartTz>: From<DateTime<FixedOffset>>,
  DateTime<FinishTz>: From<DateTime<FixedOffset>>,
{
  fn from(ser: SerializedTimespan) -> Self {
    Timespan::new(
      ser
        .start
        .map(|s| DateTime::parse_from_rfc3339(s.as_ref()).unwrap().into()),
      ser
        .finish
        .map(|s| DateTime::parse_from_rfc3339(s.as_ref()).unwrap().into()),
    )
  }
}

impl<StartTz: TimeZone, FinishTz: TimeZone> From<Timespan<StartTz, FinishTz>> for SerializedTimespan
where
  StartTz::Offset: Display,
  FinishTz::Offset: Display,
{
  fn from(timespan: Timespan<StartTz, FinishTz>) -> Self {
    SerializedTimespan {
      start: timespan.start.map(|time| time.to_rfc3339()),
      finish: timespan.finish.map(|time| time.to_rfc3339()),
    }
  }
}

impl<StartTz: TimeZone, FinishTz: TimeZone, V: Clone> From<SerializedTimespanWithValue<V>>
  for TimespanWithValue<StartTz, FinishTz, V>
where
  DateTime<StartTz>: From<DateTime<FixedOffset>>,
  DateTime<FinishTz>: From<DateTime<FixedOffset>>,
{
  fn from(ser: SerializedTimespanWithValue<V>) -> Self {
    let timespan: Timespan<StartTz, FinishTz> = SerializedTimespan {
      start: ser.start,
      finish: ser.finish,
    }
    .into();

    timespan.with_value(ser.value)
  }
}

impl<StartTz: TimeZone, FinishTz: TimeZone, V: Clone + Serialize>
  From<TimespanWithValue<StartTz, FinishTz, V>> for SerializedTimespanWithValue<V>
where
  StartTz::Offset: Display,
  FinishTz::Offset: Display,
{
  fn from(twv: TimespanWithValue<StartTz, FinishTz, V>) -> Self {
    let serialized_timespan: SerializedTimespan = twv.timespan.into();
    SerializedTimespanWithValue {
      start: serialized_timespan.start,
      finish: serialized_timespan.finish,
      value: twv.value,
    }
  }
}

impl<Tz: TimeZone + From<Utc> + std::fmt::Debug, V: Clone + Default + std::fmt::Debug>
  From<SerializedScheduledValue<V>> for ScheduledValue<Tz, V>
where
  DateTime<Tz>: From<DateTime<FixedOffset>>,
{
  fn from(ser: SerializedScheduledValue<V>) -> Self {
    ScheduledValue::from_timespans_with_values(
      Utc.into(),
      &mut ser.timespans.into_iter().map(|timespan| {
        let twv: TimespanWithValue<Tz, Tz, V> = timespan.into();
        twv.timespan.with_value(twv.value)
      }),
    )
  }
}

impl<Tz: TimeZone + From<Utc>, V: Clone + Default> From<ScheduledValue<Tz, V>>
  for SerializedScheduledValue<V>
where
  Tz::Offset: Display,
  V: Clone + Serialize,
{
  fn from(scheduled_value: ScheduledValue<Tz, V>) -> Self {
    SerializedScheduledValue {
      timespans: scheduled_value
        .into_iter()
        .filter_map(|timespan| {
          if let Some(value) = timespan.value {
            Some(timespan.timespan.with_value(value))
          } else {
            None
          }
        })
        .map(|twv| twv.into())
        .collect(),
    }
  }
}
