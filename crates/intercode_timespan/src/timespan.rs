use std::fmt::Display;
use std::ops::RangeBounds;

use crate::serialization::{SerializedTimespan, SerializedTimespanWithValue};
use chrono::{DateTime, FixedOffset, TimeZone};
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
  from = "SerializedTimespan",
  into = "SerializedTimespan",
  bound(
    serialize = "StartTz::Offset: Display, FinishTz::Offset: Display",
    deserialize = "DateTime<StartTz>: From<DateTime<FixedOffset>>, DateTime<FinishTz>: From<DateTime<FixedOffset>>"
  )
)]
pub struct Timespan<StartTz: TimeZone, FinishTz: TimeZone> {
  pub start: Option<DateTime<StartTz>>,
  pub finish: Option<DateTime<FinishTz>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
  from = "SerializedTimespanWithValue<T>",
  into = "SerializedTimespanWithValue<T>",
  bound(
    serialize = "StartTz::Offset: Display, FinishTz::Offset: Display, T: Serialize",
    deserialize = "DateTime<StartTz>: From<DateTime<FixedOffset>>, DateTime<FinishTz>: From<DateTime<FixedOffset>>, T: Deserialize<'de>"
  )
)]
pub struct TimespanWithValue<StartTz: TimeZone, FinishTz: TimeZone, T: Clone> {
  pub timespan: Timespan<StartTz, FinishTz>,
  pub value: T,
}

impl<StartTz: TimeZone, FinishTz: TimeZone> Timespan<StartTz, FinishTz> {
  pub fn new(start: Option<DateTime<StartTz>>, finish: Option<DateTime<FinishTz>>) -> Self {
    Timespan { start, finish }
  }

  pub fn contains<Tz: TimeZone>(&self, time: &DateTime<Tz>) -> bool {
    if let Some(start) = &self.start {
      if time < start {
        return false;
      }
    }

    if let Some(finish) = &self.finish {
      if time >= finish {
        return false;
      }
    }

    true
  }

  pub fn overlaps<OtherStart: TimeZone, OtherFinish: TimeZone>(
    &self,
    other: &Timespan<OtherStart, OtherFinish>,
  ) -> bool {
    if let Some(finish) = &self.finish {
      if let Some(other_start) = &other.start {
        if other_start >= finish {
          return false;
        }
      }
    }

    if let Some(start) = &self.start {
      if let Some(other_finish) = &other.finish {
        if start >= other_finish {
          return false;
        }
      }
    }

    true
  }

  pub fn with_value<'v, T: Clone + 'v>(self, value: T) -> TimespanWithValue<StartTz, FinishTz, T> {
    TimespanWithValue {
      timespan: self,
      value,
    }
  }

  pub fn with_timezone<TargetTz: TimeZone>(&self, tz: &TargetTz) -> Timespan<TargetTz, TargetTz> {
    Timespan {
      start: self.start.as_ref().map(|start| start.with_timezone(tz)),
      finish: self.finish.as_ref().map(|finish| finish.with_timezone(tz)),
    }
  }
}

impl<Tz: TimeZone> RangeBounds<DateTime<Tz>> for &Timespan<Tz, Tz> {
  fn start_bound(&self) -> std::ops::Bound<&DateTime<Tz>> {
    match &self.start {
      Some(start) => std::ops::Bound::Included(start),
      None => std::ops::Bound::Unbounded,
    }
  }

  fn end_bound(&self) -> std::ops::Bound<&DateTime<Tz>> {
    match &self.finish {
      Some(finish) => std::ops::Bound::Excluded(finish),
      None => std::ops::Bound::Unbounded,
    }
  }
}

impl<Tz: TimeZone> Timespan<Tz, Tz> {
  pub fn instant(time: DateTime<Tz>) -> Self {
    Timespan {
      start: Some(time.clone()),
      finish: Some(time),
    }
  }
}

impl<StartTz: TimeZone, FinishTz: TimeZone> Timespan<StartTz, FinishTz>
where
  <StartTz as TimeZone>::Offset: std::fmt::Display,
  <FinishTz as TimeZone>::Offset: std::fmt::Display,
{
  pub fn start_description(&self, language_loader: &FluentLanguageLoader, format: &str) -> String {
    if let Some(start) = &self.start {
      fl!(
        language_loader,
        "start_bounded",
        start = start.format(format).to_string()
      )
    } else {
      fl!(language_loader, "start_unbounded")
    }
  }

  pub fn finish_description(&self, language_loader: &FluentLanguageLoader, format: &str) -> String {
    if let Some(finish) = &self.finish {
      fl!(
        language_loader,
        "finish_bounded",
        finish = finish.format(format).to_string()
      )
    } else {
      fl!(language_loader, "finish_unbounded")
    }
  }
}

impl<StartTz: TimeZone, FinishTz: TimeZone, OtherStart: TimeZone, OtherFinish: TimeZone>
  PartialEq<Timespan<OtherStart, OtherFinish>> for Timespan<StartTz, FinishTz>
{
  fn eq(&self, other: &Timespan<OtherStart, OtherFinish>) -> bool {
    if let Some(start) = &self.start {
      if let Some(other_start) = &other.start {
        if start != &other_start.with_timezone(&start.timezone()) {
          return false;
        }
      } else {
        return false;
      }
    }

    if let Some(finish) = &self.finish {
      if let Some(other_finish) = &other.finish {
        if finish != &other_finish.with_timezone(&finish.timezone()) {
          return false;
        }
      } else {
        return false;
      }
    }

    true
  }
}

impl<
    'v,
    StartTz: TimeZone,
    FinishTz: TimeZone,
    OtherStart: TimeZone,
    OtherFinish: TimeZone,
    T: Clone + 'v,
  > PartialEq<TimespanWithValue<OtherStart, OtherFinish, T>>
  for TimespanWithValue<StartTz, FinishTz, T>
{
  fn eq(&self, other: &TimespanWithValue<OtherStart, OtherFinish, T>) -> bool {
    self.timespan.eq(&other.timespan)
  }
}

impl<
    'v,
    StartTz: TimeZone,
    FinishTz: TimeZone,
    OtherStart: TimeZone,
    OtherFinish: TimeZone,
    T: Clone + 'v,
  > PartialEq<Timespan<OtherStart, OtherFinish>> for TimespanWithValue<StartTz, FinishTz, T>
{
  fn eq(&self, other: &Timespan<OtherStart, OtherFinish>) -> bool {
    self.timespan.eq(other)
  }
}

impl<StartTz: TimeZone, FinishTz: TimeZone, OtherStart: TimeZone, OtherFinish: TimeZone>
  PartialOrd<Timespan<OtherStart, OtherFinish>> for Timespan<StartTz, FinishTz>
{
  fn partial_cmp(&self, other: &Timespan<OtherStart, OtherFinish>) -> Option<std::cmp::Ordering> {
    if self.eq(other) {
      return Some(core::cmp::Ordering::Equal);
    }

    if other.overlaps(self) {
      return None;
    }

    if let Some(finish) = &self.finish {
      if let Some(other_start) = &other.start {
        if other_start >= finish {
          return Some(core::cmp::Ordering::Less);
        }
      }
    }

    if let Some(start) = &self.start {
      if let Some(other_finish) = &other.finish {
        if other_finish <= start {
          return Some(core::cmp::Ordering::Greater);
        }
      }
    }

    None
  }
}

impl<
    'v,
    StartTz: TimeZone,
    FinishTz: TimeZone,
    OtherStart: TimeZone,
    OtherFinish: TimeZone,
    T: 'v + Clone,
  > PartialOrd<TimespanWithValue<OtherStart, OtherFinish, T>>
  for TimespanWithValue<StartTz, FinishTz, T>
{
  fn partial_cmp(
    &self,
    other: &TimespanWithValue<OtherStart, OtherFinish, T>,
  ) -> Option<std::cmp::Ordering> {
    self.timespan.partial_cmp(&other.timespan)
  }
}

impl<
    'v,
    StartTz: TimeZone,
    FinishTz: TimeZone,
    OtherStart: TimeZone,
    OtherFinish: TimeZone,
    T: 'v + Clone,
  > PartialOrd<Timespan<OtherStart, OtherFinish>> for TimespanWithValue<StartTz, FinishTz, T>
{
  fn partial_cmp(&self, other: &Timespan<OtherStart, OtherFinish>) -> Option<std::cmp::Ordering> {
    self.timespan.partial_cmp(other)
  }
}

impl<StartTz: TimeZone, FinishTz: TimeZone> Eq for Timespan<StartTz, FinishTz> {}

impl<'v, StartTz: TimeZone, FinishTz: TimeZone, T: 'v + Clone> Eq
  for TimespanWithValue<StartTz, FinishTz, T>
{
}

impl<StartTz: TimeZone, FinishTz: TimeZone> Ord for Timespan<StartTz, FinishTz> {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    if self.eq(other) {
      core::cmp::Ordering::Equal
    } else {
      self.start.cmp(&other.start)
    }
  }
}

impl<'v, StartTz: TimeZone, FinishTz: TimeZone, T: 'v + Clone> Ord
  for TimespanWithValue<StartTz, FinishTz, T>
{
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.timespan.cmp(&other.timespan)
  }
}
