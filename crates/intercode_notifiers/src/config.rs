use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

fn default_sends_sms() -> bool {
  false
}

trait Keyed {
  fn get_key(&self) -> &str;

  fn deserialize_keyed<'de, D>(deserializer: D) -> Result<HashMap<String, Self>, D::Error>
  where
    Self: Sized + Deserialize<'de>,
    D: Deserializer<'de>,
  {
    let items: Vec<Self> = Deserialize::deserialize(deserializer)?;
    Ok(
      items
        .into_iter()
        .map(|item| (item.get_key().to_owned(), item))
        .collect(),
    )
  }

  fn serialize_keyed<S>(items: &HashMap<String, Self>, serializer: S) -> Result<S::Ok, S::Error>
  where
    Self: Sized + Serialize,
    S: Serializer,
  {
    serializer.collect_seq(items.values())
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationEventConfig {
  pub key: String,
  pub name: String,
  pub destination_description: String,
  #[serde(default = "default_sends_sms")]
  pub sends_sms: bool,
}

impl Keyed for NotificationEventConfig {
  fn get_key(&self) -> &str {
    &self.key
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationCategoryConfig {
  pub key: String,
  pub name: String,
  #[serde(
    serialize_with = "Keyed::serialize_keyed",
    deserialize_with = "Keyed::deserialize_keyed"
  )]
  pub events: HashMap<String, NotificationEventConfig>,
}

impl Keyed for NotificationCategoryConfig {
  fn get_key(&self) -> &str {
    &self.key
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationsConfig {
  #[serde(
    serialize_with = "Keyed::serialize_keyed",
    deserialize_with = "Keyed::deserialize_keyed"
  )]
  pub categories: HashMap<String, NotificationCategoryConfig>,
}

pub static NOTIFICATIONS_CONFIG: Lazy<NotificationsConfig> =
  Lazy::new(|| serde_json::from_str(include_str!("../../../config/notifications.json")).unwrap());
