use std::{
  fmt::Debug,
  sync::atomic::{AtomicI64, Ordering},
};

use async_graphql::indexmap::IndexMap;
use chrono::TimeZone;
use intercode_timespan::TimespanWithValue;
use liquid::{model::DateTime, ValueView};
use once_cell::race::OnceBox;
use seawater::liquid_drop_impl;
use serde::Serialize;

use super::{utils::date_time_to_liquid_date_time, DropContext};

#[derive(Debug)]
pub struct TimespanWithValueDrop<
  StartTz: TimeZone + Debug,
  FinishTz: TimeZone + Debug,
  V: Clone + Serialize + Debug,
> {
  pub timespan_with_value: TimespanWithValue<StartTz, FinishTz, V>,
  context: DropContext,
  id: i64,
  _liquid_object_view_pairs: OnceBox<IndexMap<String, Box<dyn ValueView + Send + Sync>>>,
}

impl<StartTz: TimeZone + Debug, FinishTz: TimeZone + Debug, V: Clone + Serialize + Debug> Clone
  for TimespanWithValueDrop<StartTz, FinishTz, V>
{
  fn clone(&self) -> Self {
    Self {
      timespan_with_value: self.timespan_with_value.clone(),
      context: self.context.clone(),
      id: self.id,
      _liquid_object_view_pairs: OnceBox::new(),
    }
  }
}

static NEXT_ID: AtomicI64 = AtomicI64::new(0);

#[liquid_drop_impl(i64, DropContext)]
impl<
    StartTz: TimeZone + Debug + Send + Sync + 'static,
    FinishTz: TimeZone + Debug + Send + Sync + 'static,
    V: Clone + Serialize + Debug + Send + Sync + 'static,
  > TimespanWithValueDrop<StartTz, FinishTz, V>
where
  StartTz::Offset: Send + Sync,
  FinishTz::Offset: Send + Sync,
{
  pub fn new(
    timespan_with_value: TimespanWithValue<StartTz, FinishTz, V>,
    context: DropContext,
  ) -> Self {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    TimespanWithValueDrop {
      id,
      timespan_with_value,
      context,
      _liquid_object_view_pairs: OnceBox::new(),
    }
  }

  fn id(&self) -> i64 {
    self.id
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

impl<
    StartTz: TimeZone + Debug + Send + Sync + 'static,
    FinishTz: TimeZone + Debug + Send + Sync + 'static,
    V: Clone + Serialize + Debug + Send + Sync + 'static,
  > From<TimespanWithValueDrop<StartTz, FinishTz, Option<V>>>
  for Option<TimespanWithValueDrop<StartTz, FinishTz, V>>
where
  StartTz::Offset: Send + Sync,
  FinishTz::Offset: Send + Sync,
{
  fn from(drop: TimespanWithValueDrop<StartTz, FinishTz, Option<V>>) -> Self {
    match drop.timespan_with_value.value {
      Some(value) => Some(TimespanWithValueDrop::new(
        TimespanWithValue {
          value,
          timespan: drop.timespan_with_value.timespan,
        },
        drop.context.clone(),
      )),
      None => None,
    }
  }
}
