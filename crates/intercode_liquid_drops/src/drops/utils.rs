use chrono::{NaiveDateTime, TimeZone};
use liquid::model::DateTime;

pub fn date_time_to_liquid_date_time<Tz: TimeZone>(t: &chrono::DateTime<Tz>) -> Option<DateTime> {
  naive_date_time_to_liquid_date_time(t.naive_utc())
}

pub fn naive_date_time_to_liquid_date_time(t: NaiveDateTime) -> Option<DateTime> {
  DateTime::from_str(t.format("%Y-%m-%d %H:%M:%S").to_string().as_str())
}
