use std::{collections::BTreeMap, fmt::Debug};

use crate::{serialization::SerializedScheduledValue, Timespan, TimespanWithValue};
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(
  from = "SerializedScheduledValue<V>",
  bound(
    deserialize = "V: Deserialize<'de> + std::fmt::Debug, Tz: From<Utc> + std::fmt::Debug, DateTime<Tz>: From<DateTime<FixedOffset>>"
  )
)]
pub struct ScheduledValue<Tz: TimeZone, V: Clone + Default> {
  values_by_start: BTreeMap<DateTime<Tz>, Option<V>>,
  start_value: Option<V>,
  finish_value: Option<V>,
  tz: Tz,
}

impl<Tz: TimeZone + std::fmt::Debug, V: Clone + std::fmt::Debug + Default> ScheduledValue<Tz, V> {
  pub fn new(default_value: Option<V>, tz: Tz) -> Self {
    ScheduledValue {
      values_by_start: BTreeMap::new(),
      start_value: default_value.clone(),
      finish_value: default_value,
      tz,
    }
  }

  pub fn from_timespans_with_values(
    tz: Tz,
    timespans_with_values: &mut dyn Iterator<Item = TimespanWithValue<Tz, Tz, V>>,
  ) -> Self {
    let mut scheduled_value = ScheduledValue::new(None, tz);

    timespans_with_values.for_each(|timespan_with_value| {
      scheduled_value.add(&timespan_with_value.timespan, timespan_with_value.value);
    });

    scheduled_value
  }

  pub fn add(&mut self, timespan: &Timespan<Tz, Tz>, value: V) {
    let mut range = self.values_by_start.range(timespan);
    let first_start = range.next();
    if let Some(first_start) = first_start {
      if let Some(start) = &timespan.start {
        if start < first_start.0 {
          panic!("Tried to insert overlapping timespan into scheduled value");
        }
      }
    }

    if let Some(start) = &timespan.start {
      self
        .values_by_start
        .insert(start.to_owned(), Some(value.to_owned()));
    } else {
      self.start_value = Some(value.to_owned());
    }

    if let Some(finish) = &timespan.finish {
      self.values_by_start.insert(finish.to_owned(), None);
    } else {
      self.finish_value = Some(value);
    }
  }

  pub fn timespan_containing<OtherTz: TimeZone>(
    &self,
    time: DateTime<OtherTz>,
  ) -> Option<TimespanWithValue<Tz, Tz, V>> {
    let mut prev = self
      .values_by_start
      .range(..time.with_timezone(&self.tz))
      .next_back();

    let mut forward = self.values_by_start.range(time.with_timezone(&self.tz)..);
    let mut next = forward.next();
    if let Some((next_change, next_value)) = next {
      if next_change == &time {
        prev = Some((next_change, next_value));
        next = forward.next();
      }
    }

    if let Some(prev) = prev {
      prev.1.as_ref().map(|value| {
        Timespan::new(Some(prev.0.to_owned()), next.map(|next| next.0.to_owned()))
          .with_value(value.to_owned())
      })
    } else {
      self.start_value.as_ref().map(|start_value| {
        Timespan::new(None, next.map(|next| next.0.to_owned())).with_value(start_value.to_owned())
      })
    }
  }

  pub fn timespan_overlapping<OtherStart: TimeZone, OtherFinish: TimeZone>(
    &self,
    other: &Timespan<OtherStart, OtherFinish>,
  ) -> Option<TimespanWithValue<Tz, Tz, Option<V>>> {
    self
      .into_iter()
      .find(|timespan| timespan.timespan.overlaps(other))
  }

  pub fn value_at<OtherTz: TimeZone>(&self, time: DateTime<OtherTz>) -> Option<V> {
    self.timespan_containing(time).map(|twv| twv.value)
  }

  pub fn current_value_changed_at<OtherTz: TimeZone>(
    &self,
    time: DateTime<OtherTz>,
  ) -> Option<&DateTime<Tz>> {
    let mut range = self.values_by_start.range(..time.with_timezone(&self.tz));

    let item = range.next_back();
    item.map(|item| item.0)
  }

  pub fn next_value_change_after<OtherTz: TimeZone>(
    &self,
    time: DateTime<OtherTz>,
  ) -> Option<&DateTime<Tz>> {
    let mut range = self.values_by_start.range(time.with_timezone(&self.tz)..);

    let item = range.next();
    if let Some(item) = item {
      if item.0 == &time {
        return range.next().map(|next| next.0);
      }
    }

    item.map(|item| item.0)
  }
}

impl<Tz: TimeZone + From<Utc> + std::fmt::Debug, V: Clone + Default + std::fmt::Debug> Default
  for ScheduledValue<Tz, V>
{
  fn default() -> Self {
    ScheduledValue::new(Default::default(), Utc.into())
  }
}

impl<StartTz: TimeZone + Debug, FinishTz: TimeZone + Debug, V: Clone + Default + Debug>
  FromIterator<TimespanWithValue<StartTz, FinishTz, V>> for ScheduledValue<Utc, V>
{
  fn from_iter<T: IntoIterator<Item = TimespanWithValue<StartTz, FinishTz, V>>>(iter: T) -> Self {
    ScheduledValue::from_timespans_with_values(
      Utc,
      &mut iter.into_iter().map(|twv| TimespanWithValue {
        timespan: twv.timespan.with_timezone(&Utc),
        value: twv.value,
      }),
    )
  }
}

pub struct ScheduledValueIterator<'a, Tz: TimeZone, V: Clone + Default> {
  output_start: bool,
  output_finish: bool,
  prev_start: DateTime<Tz>,
  prev_value: Option<V>,
  finish_value: Option<&'a V>,
  values_by_start_iterator: Box<dyn Iterator<Item = (&'a DateTime<Tz>, &'a Option<V>)> + 'a>,
}

impl<'a, Tz: TimeZone, V: Clone + Default> Iterator for ScheduledValueIterator<'a, Tz, V> {
  type Item = TimespanWithValue<Tz, Tz, Option<V>>;

  fn next(&mut self) -> Option<Self::Item> {
    if !self.output_start {
      self.output_start = true;
      if let Some(prev_value) = &self.prev_value {
        let timespan = Timespan::new(None, Some(self.prev_start.to_owned()))
          .with_value(Some(prev_value.to_owned()));
        return Some(timespan);
      } else {
        self.output_finish = true;
        return Some(Timespan::new(None, None).with_value(self.finish_value.map(|v| v.to_owned())));
      }
    }

    let next_item = self.values_by_start_iterator.next();
    if let Some((next_start, next_value)) = next_item {
      let timespan = Timespan::new(
        Some(self.prev_start.to_owned()),
        Some(next_start.to_owned()),
      )
      .with_value(self.prev_value.to_owned());
      self.prev_start = next_start.to_owned();
      self.prev_value = next_value.to_owned();
      return Some(timespan);
    }

    if !self.output_finish {
      self.output_finish = true;
      return Some(
        Timespan::new(Some(self.prev_start.to_owned()), None)
          .with_value(self.prev_value.to_owned()),
      );
    }

    None
  }
}

impl<'a, Tz: TimeZone, V: Clone + Default> IntoIterator for &'a ScheduledValue<Tz, V> {
  type Item = TimespanWithValue<Tz, Tz, Option<V>>;
  type IntoIter = ScheduledValueIterator<'a, Tz, V>;

  fn into_iter(self) -> Self::IntoIter {
    let mut values_by_start = self.values_by_start.iter().peekable();
    let first_change = values_by_start.peek().to_owned();

    ScheduledValueIterator {
      output_start: false,
      output_finish: false,
      prev_start: first_change
        .map(|item| item.0.to_owned())
        .unwrap_or_else(|| Utc::now().with_timezone(&self.tz)), // doesn't matter, we just need to put something here
      prev_value: first_change.and_then(|item| item.1.to_owned()),
      finish_value: self.finish_value.as_ref(),
      values_by_start_iterator: Box::new(values_by_start),
    }
  }
}
