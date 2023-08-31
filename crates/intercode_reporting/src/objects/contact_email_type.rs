use async_graphql::Object;
use intercode_graphql_core::scalars::JsonScalar;
use mailparse::SingleInfo;
use serde_json::Value;

#[derive(Clone)]
pub struct ContactEmail {
  pub email: String,
  pub name: String,
  pub address_name: Option<String>,
  pub metadata: serde_json::Map<String, Value>,
}

impl ContactEmail {
  pub fn new(
    email: String,
    name: String,
    address_name: Option<String>,
    metadata: impl IntoIterator<Item = (String, Value)>,
  ) -> Self {
    ContactEmail {
      email,
      name,
      address_name,
      metadata: serde_json::Map::from_iter(metadata),
    }
  }
}

#[derive(Clone)]
pub struct ContactEmailType(pub ContactEmail);

#[Object(name = "ContactEmail")]
impl ContactEmailType {
  pub async fn email(&self) -> &str {
    &self.0.email
  }

  #[graphql(name = "formatted_address")]
  pub async fn formatted_address(&self) -> String {
    format!(
      "{}",
      SingleInfo {
        addr: self.0.email.clone(),
        display_name: Some(
          self
            .0
            .address_name
            .clone()
            .unwrap_or_else(|| self.0.name.clone())
        )
      }
    )
  }

  #[graphql(name = "metadata_json")]
  pub async fn metadata_json(&self) -> JsonScalar {
    JsonScalar(Value::Object(self.0.metadata.clone()))
  }

  pub async fn name(&self) -> &str {
    &self.0.name
  }
}
