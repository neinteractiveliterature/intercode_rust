use async_graphql::{InputValueError, Scalar, ScalarType};
use chrono::NaiveDateTime;

pub struct DateScalar(pub NaiveDateTime);

#[Scalar]
impl ScalarType for DateScalar {
  fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
    NaiveDateTime::parse(value)
      .map(DateScalar)
      .map_err(InputValueError::propagate)
  }

  fn to_value(&self) -> async_graphql::Value {
    NaiveDateTime::to_value(&self.0)
  }
}
