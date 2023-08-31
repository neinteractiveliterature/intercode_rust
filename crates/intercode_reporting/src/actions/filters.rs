use chrono::NaiveDateTime;
use intercode_inflector::IntercodeInflector;

pub fn format_run_day_and_time(datetime: Option<NaiveDateTime>) -> Result<String, askama::Error> {
  Ok(
    datetime
      .map(|time| time.format("%a %l:%M%P").to_string())
      .unwrap_or_default(),
  )
}

pub fn pluralize(
  count: i64,
  name: &str,
  inflector: &IntercodeInflector,
) -> Result<String, askama::Error> {
  if count == 1 {
    Ok(format!("{}{}", count, name))
  } else {
    Ok(format!("{}{}", count, inflector.pluralize(name)))
  }
}
