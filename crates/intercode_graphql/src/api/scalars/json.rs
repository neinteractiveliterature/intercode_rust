use async_graphql::{Scalar, ScalarType};

pub struct JsonScalar(serde_json::Value);

#[Scalar]
impl ScalarType for JsonScalar {
  fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
    Ok(JsonScalar(value.into_json()?))
  }

  fn to_value(&self) -> async_graphql::Value {
    async_graphql::Value::from_json(self.0.clone()).unwrap()
  }
}
