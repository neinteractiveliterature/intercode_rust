use chrono::Utc;
use i18n_embed::fluent::FluentLanguageLoader;
use intercode_entities::conventions;
use intercode_timespan::ScheduledValue;
use liquid::model::DateTime;
use sea_orm::JsonValue;
use serde::{
  de::{self, Unexpected},
  Deserialize, Deserializer, Serialize, Serializer,
};

use super::{utils::naive_date_time_to_liquid_date_time, ScheduledValueDrop};

#[derive(Clone, Debug)]
pub enum MaximumEventSignupsValue {
  Unlimited,
  NotYet,
  NotNow,
  Limited(u16),
}

impl Serialize for MaximumEventSignupsValue {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      MaximumEventSignupsValue::Unlimited => serializer.serialize_str("unlimited"),
      MaximumEventSignupsValue::NotYet => serializer.serialize_str("not_yet"),
      MaximumEventSignupsValue::NotNow => serializer.serialize_str("not_now"),
      MaximumEventSignupsValue::Limited(num) => {
        serializer.serialize_str(format!("{}", num).as_str())
      }
    }
  }
}

impl<'de> Deserialize<'de> for MaximumEventSignupsValue {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct MaximumEventSignupsVisitor;

    impl<'de> de::Visitor<'de> for MaximumEventSignupsVisitor {
      type Value = MaximumEventSignupsValue;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter
          .write_str("unlimited, not_yet, not_now, or a number of signups allowed at this time")
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        match v {
          "unlimited" => Ok(MaximumEventSignupsValue::Unlimited),
          "not_yet" => Ok(MaximumEventSignupsValue::NotYet),
          "not_now" => Ok(MaximumEventSignupsValue::NotNow),
          _ => v
            .parse()
            .map(|num| MaximumEventSignupsValue::Limited(num))
            .map_err(|_e| de::Error::invalid_value(Unexpected::Str(v), &self)),
        }
      }
    }

    deserializer.deserialize_str(MaximumEventSignupsVisitor)
  }
}

impl From<MaximumEventSignupsValue> for u16 {
  fn from(value: MaximumEventSignupsValue) -> Self {
    match value {
      MaximumEventSignupsValue::Unlimited => u16::MAX,
      MaximumEventSignupsValue::NotYet => 0,
      MaximumEventSignupsValue::NotNow => 0,
      MaximumEventSignupsValue::Limited(num) => num,
    }
  }
}

impl Default for MaximumEventSignupsValue {
  fn default() -> Self {
    MaximumEventSignupsValue::NotYet
  }
}

#[derive(Serialize)]
pub struct ConventionDrop<'a> {
  id: i64,
  name: Option<&'a str>,
  location: Option<&'a JsonValue>,
  maximum_event_signups: ScheduledValueDrop<MaximumEventSignupsValue>,
  starts_at: Option<DateTime>,
  ends_at: Option<DateTime>,
}

impl<'a> ConventionDrop<'a> {
  pub fn new(
    convention: &'a conventions::Model,
    language_loader: &'a FluentLanguageLoader,
  ) -> Self {
    ConventionDrop {
      id: convention.id,
      name: convention.name.as_deref(),
      location: convention.location.as_ref(),
      maximum_event_signups: convention
        .maximum_event_signups
        .as_ref()
        .map(|maximum_event_signups| {
          let scheduled_value: ScheduledValue<Utc, MaximumEventSignupsValue> =
            serde_json::from_value(maximum_event_signups.clone()).unwrap_or_default();
          ScheduledValueDrop::new(scheduled_value, language_loader)
        })
        .unwrap_or_else(|| ScheduledValueDrop::new::<Utc>(Default::default(), language_loader)),
      starts_at: convention
        .starts_at
        .and_then(naive_date_time_to_liquid_date_time),
      ends_at: convention
        .ends_at
        .and_then(naive_date_time_to_liquid_date_time),
    }
  }
}
