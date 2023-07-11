use std::fmt::Display;

use async_graphql::{InputValueError, Scalar, ScalarType};
use chrono::{DateTime, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

pub struct DateScalar<Tz: TimeZone = Utc>(pub DateTime<Tz>)
where
  Tz::Offset: Send + Sync;

#[Scalar(name = "Date")]
impl ScalarType for DateScalar<Utc> {
  fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
    DateTime::<Utc>::parse(value)
      .map(DateScalar)
      .map_err(InputValueError::propagate)
  }

  fn to_value(&self) -> async_graphql::Value {
    DateTime::<Utc>::to_value(&self.0)
  }
}

impl<Tz: TimeZone> From<DateScalar<Tz>> for NaiveDateTime
where
  Tz::Offset: Send + Sync,
{
  fn from(scalar: DateScalar<Tz>) -> Self {
    scalar.0.naive_utc()
  }
}

impl<Tz: TimeZone> From<DateScalar<Tz>> for NaiveDate
where
  Tz::Offset: Send + Sync,
{
  fn from(scalar: DateScalar<Tz>) -> Self {
    scalar.0.naive_utc().date()
  }
}

impl<Tz: TimeZone> TryFrom<LocalResult<DateTime<Tz>>> for DateScalar<Tz>
where
  Tz::Offset: Send + Sync + Display,
{
  type Error = async_graphql::Error;

  fn try_from(value: LocalResult<DateTime<Tz>>) -> Result<Self, Self::Error> {
    match value {
      LocalResult::None => Err(async_graphql::Error::new(format!(
        "No such local time in {}",
        std::any::type_name::<Tz>()
      ))),
      LocalResult::Single(t) => Ok(DateScalar(t)),
      LocalResult::Ambiguous(earliest, latest) => Err(async_graphql::Error::new(format!(
        "Ambiguous local time in {} ranging from {} to {}",
        std::any::type_name::<Tz>(),
        earliest,
        latest
      ))),
    }
  }
}

impl TryFrom<NaiveDateTime> for DateScalar<Utc> {
  type Error = async_graphql::Error;

  fn try_from(value: NaiveDateTime) -> Result<Self, Self::Error> {
    value.and_local_timezone(Utc).try_into()
  }
}

impl TryFrom<NaiveDate> for DateScalar<Utc> {
  type Error = async_graphql::Error;

  fn try_from(value: NaiveDate) -> Result<Self, Self::Error> {
    value.and_time(NaiveTime::MIN).try_into()
  }
}
