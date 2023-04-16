
use chrono::NaiveDateTime;

pub fn format_run_day_and_time(datetime: Option<NaiveDateTime>) -> Result<String, askama::Error> {
  Ok(
    datetime
      .map(|time| time.format("%a %l:%M%P").to_string())
      .unwrap_or_default(),
  )
}
