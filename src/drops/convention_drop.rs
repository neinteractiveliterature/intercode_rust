use std::sync::Arc;

use chrono::Utc;
use i18n_embed::fluent::FluentLanguageLoader;
use intercode_entities::conventions;
use intercode_graphql::SchemaData;
use intercode_timespan::ScheduledValue;
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use liquid::model::ValueView;
use serde::{
  de::{self, Unexpected},
  Deserialize, Deserializer, Serialize, Serializer,
};

use super::{utils::naive_date_time_to_liquid_date_time, EventsCreatedSince, ScheduledValueDrop};

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
            .map(MaximumEventSignupsValue::Limited)
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

#[liquid_drop_struct]
pub struct ConventionDrop {
  schema_data: SchemaData,
  convention: conventions::Model,
  events_created_since: EventsCreatedSince,
  language_loader: Arc<FluentLanguageLoader>,
}

#[liquid_drop_impl]
impl ConventionDrop {
  pub fn new(
    schema_data: SchemaData,
    convention: conventions::Model,
    language_loader: Arc<FluentLanguageLoader>,
  ) -> Self {
    let convention_id = convention.id;

    ConventionDrop {
      schema_data: schema_data.clone(),
      convention,
      language_loader,
      events_created_since: EventsCreatedSince::new(schema_data, convention_id),
    }
  }

  fn id(&self) -> i64 {
    self.convention.id
  }

  fn name(&self) -> Option<&str> {
    self.convention.name.as_deref()
  }

  fn events_created_since(&self) -> &dyn ValueView {
    &self.events_created_since
  }

  #[drop(serialize_value = true)]
  fn location(&self) -> Option<&JsonValue> {
    self.convention.location.as_ref()
  }

  #[drop(serialize_value = true)]
  fn maximum_event_signups(&self) -> ScheduledValueDrop<MaximumEventSignupsValue> {
    self
      .convention
      .maximum_event_signups
      .as_ref()
      .map(|maximum_event_signups| {
        let scheduled_value: ScheduledValue<Utc, MaximumEventSignupsValue> =
          serde_json::from_value(maximum_event_signups.clone()).unwrap_or_default();
        ScheduledValueDrop::new(scheduled_value, self.language_loader.as_ref())
      })
      .unwrap_or_else(|| {
        ScheduledValueDrop::new::<Utc>(Default::default(), self.language_loader.as_ref())
      })
  }

  #[drop(serialize_value = true)]
  fn starts_at(&self) -> Option<liquid::model::DateTime> {
    self
      .convention
      .starts_at
      .and_then(naive_date_time_to_liquid_date_time)
  }

  #[drop(serialize_value = true)]
  fn ends_at(&self) -> Option<liquid::model::DateTime> {
    self
      .convention
      .ends_at
      .and_then(naive_date_time_to_liquid_date_time)
  }

  fn ticket_name(&self) -> &str {
    self.convention.ticket_name.as_str()
  }
}
