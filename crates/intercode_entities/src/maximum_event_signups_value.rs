use serde::{
  de::{self, Unexpected},
  Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Clone, Debug, Default)]
pub enum MaximumEventSignupsValue {
  Unlimited,
  #[default]
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
