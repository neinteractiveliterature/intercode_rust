use std::str::FromStr;

use async_graphql::{InputValueError, Scalar, ScalarType};
use sea_orm::prelude::{BigDecimal, Decimal};

pub struct BigDecimalScalar(BigDecimal);

#[Scalar(name = "BigDecimal")]
impl ScalarType for BigDecimalScalar {
  fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
    BigDecimal::from_str(&value.to_string())
      .map(BigDecimalScalar)
      .map_err(|err| InputValueError::custom(err.to_string()))
  }

  fn to_value(&self) -> async_graphql::Value {
    async_graphql::Value::String(self.0.to_string())
  }
}

impl TryFrom<&str> for BigDecimalScalar {
  type Error = InputValueError<BigDecimalScalar>;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    <BigDecimalScalar as ScalarType>::parse(value.into())
  }
}

impl TryFrom<String> for BigDecimalScalar {
  type Error = InputValueError<BigDecimalScalar>;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    <BigDecimalScalar as ScalarType>::parse(value.into())
  }
}

impl From<BigDecimal> for BigDecimalScalar {
  fn from(value: BigDecimal) -> Self {
    BigDecimalScalar(value)
  }
}

impl TryFrom<Decimal> for BigDecimalScalar {
  type Error = InputValueError<BigDecimalScalar>;

  fn try_from(value: Decimal) -> Result<Self, Self::Error> {
    value.to_string().try_into()
  }
}
