use async_graphql::{InputValueError, Scalar, ScalarType};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

pub struct DateScalar(pub NaiveDateTime);

#[Scalar(name = "Date")]
impl ScalarType for DateScalar {
  fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
    DateTime::<Utc>::parse(value)
      .map(|dt| dt.naive_utc())
      .map(DateScalar)
      .map_err(InputValueError::propagate)
  }

  fn to_value(&self) -> async_graphql::Value {
    NaiveDateTime::to_value(&self.0)
  }
}

impl From<DateScalar> for NaiveDateTime {
  fn from(scalar: DateScalar) -> Self {
    scalar.0
  }
}

impl From<DateScalar> for NaiveDate {
  fn from(scalar: DateScalar) -> Self {
    scalar.0.date()
  }
}

impl From<NaiveDateTime> for DateScalar {
  fn from(date: NaiveDateTime) -> Self {
    DateScalar(date)
  }
}

impl From<NaiveDate> for DateScalar {
  fn from(date: NaiveDate) -> Self {
    DateScalar(date.and_time(NaiveTime::MIN))
  }
}
