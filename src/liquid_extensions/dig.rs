use liquid::{
  model::{ArrayView, ScalarCow},
  Error, ObjectView,
};

use super::invalid_input;

pub fn get_object_from_value<'a>(
  value: &'a dyn ObjectView,
  key: &str,
  tag_name: &'static str,
  source: &str,
) -> Result<&'a dyn ObjectView, Error> {
  value
    .get(key)
    .ok_or_else(|| {
      Error::with_msg(format!("must contain an object called {}", key))
        .context(tag_name, &source.to_string())
    })?
    .as_object()
    .ok_or_else(|| {
      Error::with_msg(format!("must contain an object called {}", key))
        .context(tag_name, &source.to_string())
    })
}

pub fn get_array_from_value<'a>(
  value: &'a dyn ObjectView,
  key: &str,
  tag_name: &'static str,
  source: &str,
) -> Result<&'a dyn ArrayView, Error> {
  value
    .get(key)
    .ok_or_else(|| {
      Error::with_msg(format!("must contain an array called {}", key))
        .context(tag_name, &source.to_string())
    })?
    .as_array()
    .ok_or_else(|| {
      Error::with_msg(format!("must contain an array called {}", key))
        .context(tag_name, &source.to_string())
    })
}

pub fn liquid_datetime_to_chrono_datetime(
  input: &liquid_core::model::DateTime,
) -> chrono::DateTime<chrono::FixedOffset> {
  use crate::chrono::TimeZone;

  let offset = chrono::FixedOffset::east(input.offset().whole_seconds());
  offset
    .ymd(input.year(), input.month().into(), input.day().into())
    .and_hms_nano(
      input.hour().into(),
      input.minute().into(),
      input.second().into(),
      input.nanosecond(),
    )
}

pub fn get_scalar_from_value<'a>(
  value: &'a dyn ObjectView,
  key: &str,
  tag_name: &'static str,
  source: &str,
) -> Result<ScalarCow<'a>, Error> {
  value
    .get(key)
    .ok_or_else(|| {
      Error::with_msg(format!("must contain a value called {}", key))
        .context(tag_name, &source.to_string())
    })?
    .as_scalar()
    .ok_or_else(|| {
      Error::with_msg(format!("must contain a value called {}", key))
        .context(tag_name, &source.to_string())
    })
}

pub fn get_datetime_from_value<'a>(
  value: &'a dyn ObjectView,
  key: &str,
  tag_name: &'static str,
  source: &str,
) -> Result<chrono::DateTime<chrono::FixedOffset>, Error> {
  let scalar = get_scalar_from_value(value, key, tag_name, source)?;

  scalar
    .to_date_time()
    .map(|dt| liquid_datetime_to_chrono_datetime(&dt))
    .ok_or_else(|| invalid_input(format!("Cannot parse {} as datetime", key)))
}

pub fn dig_value<'a>(
  value: &'a dyn ObjectView,
  keys: Vec<&str>,
  tag_name: &'static str,
  source: &str,
) -> Result<ScalarCow<'a>, Error> {
  let (value_key, object_keys) = keys.split_last().ok_or_else(|| {
    Error::with_msg("dig_value requires at least one key").context(tag_name, &source.to_string())
  })?;

  let object = object_keys.iter().try_fold(value, |acc, object_key| {
    get_object_from_value(acc, object_key, tag_name, source)
  })?;

  get_scalar_from_value(object, value_key, tag_name, source)
}
