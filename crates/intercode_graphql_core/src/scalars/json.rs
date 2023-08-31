use async_graphql::{Scalar, ScalarType};

pub struct JsonScalar(pub serde_json::Value);

#[Scalar(name = "Json")]
impl ScalarType for JsonScalar {
  fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
    match value {
      async_graphql::Value::Null => Ok(JsonScalar(serde_json::Value::Null)),
      async_graphql::Value::String(json) => serde_json::from_str(&json)
        .map(JsonScalar)
        .map_err(|err| err.into()),
      _ => value.into_json().map(JsonScalar).map_err(|err| err.into()),
    }
  }

  fn to_value(&self) -> async_graphql::Value {
    async_graphql::Value::String(serde_json::to_string(&self.0).unwrap())
  }
}
