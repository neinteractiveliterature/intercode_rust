use chrono::{DateTime, TimeZone};
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;

#[derive(Debug)]
pub struct Timespan<StartTz: TimeZone, FinishTz: TimeZone> {
  pub start: Option<DateTime<StartTz>>,
  pub finish: Option<DateTime<FinishTz>>,
}

#[derive(Debug)]
pub struct TimespanWithValue<StartTz: TimeZone, FinishTz: TimeZone, T> {
  pub timespan: Timespan<StartTz, FinishTz>,
  pub value: T,
}

impl<StartTz: TimeZone, FinishTz: TimeZone> Timespan<StartTz, FinishTz> {
  pub fn contains<Tz: TimeZone>(&self, time: &DateTime<Tz>) -> bool {
    if let Some(start) = &self.start {
      if time < start {
        return false;
      }
    }

    if let Some(finish) = &self.finish {
      if time >= finish {
        return false;
      }
    }

    true
  }

  pub fn overlaps<OtherStart: TimeZone, OtherFinish: TimeZone>(
    &self,
    other: &Timespan<OtherStart, OtherFinish>,
  ) -> bool {
    if let Some(finish) = &self.finish {
      if let Some(other_start) = &other.start {
        if other_start >= finish {
          return false;
        }
      }
    }

    if let Some(start) = &self.start {
      if let Some(other_finish) = &other.finish {
        if start >= other_finish {
          return false;
        }
      }
    }

    true
  }

  pub fn with_value<T>(self, value: T) -> TimespanWithValue<StartTz, FinishTz, T> {
    TimespanWithValue {
      timespan: self,
      value,
    }
  }

  pub fn with_timezone<TargetTz: TimeZone>(&self, tz: &TargetTz) -> Timespan<TargetTz, TargetTz> {
    Timespan {
      start: (&self.start.as_ref()).and_then(|start| Some(start.with_timezone(tz))),
      finish: (&self.finish.as_ref()).and_then(|finish| Some(finish.with_timezone(tz))),
    }
  }
}

impl<StartTz: TimeZone, FinishTz: TimeZone> Timespan<StartTz, FinishTz>
where
  <StartTz as TimeZone>::Offset: std::fmt::Display,
  <FinishTz as TimeZone>::Offset: std::fmt::Display,
{
  pub fn start_description(&self, language_loader: &FluentLanguageLoader, format: &str) -> String {
    if let Some(start) = &self.start {
      fl!(
        language_loader,
        "start_bounded",
        start = start.format(format).to_string()
      )
    } else {
      fl!(language_loader, "start_unbounded")
    }
  }

  pub fn finish_description(&self, language_loader: &FluentLanguageLoader, format: &str) -> String {
    if let Some(finish) = &self.finish {
      fl!(
        language_loader,
        "finish_bounded",
        finish = finish.format(format).to_string()
      )
    } else {
      fl!(language_loader, "finish_unbounded")
    }
  }
}

impl<StartTz: TimeZone, FinishTz: TimeZone, OtherStart: TimeZone, OtherFinish: TimeZone>
  PartialEq<Timespan<OtherStart, OtherFinish>> for Timespan<StartTz, FinishTz>
{
  fn eq(&self, other: &Timespan<OtherStart, OtherFinish>) -> bool {
    if let Some(start) = &self.start {
      if let Some(other_start) = &other.start {
        if start != &other_start.with_timezone(&start.timezone()) {
          return false;
        }
      } else {
        return false;
      }
    }

    if let Some(finish) = &self.finish {
      if let Some(other_finish) = &other.finish {
        if finish != &other_finish.with_timezone(&finish.timezone()) {
          return false;
        }
      } else {
        return false;
      }
    }

    true
  }
}

impl<StartTz: TimeZone, FinishTz: TimeZone, OtherStart: TimeZone, OtherFinish: TimeZone>
  PartialOrd<Timespan<OtherStart, OtherFinish>> for Timespan<StartTz, FinishTz>
{
  fn partial_cmp(&self, other: &Timespan<OtherStart, OtherFinish>) -> Option<std::cmp::Ordering> {
    if self.eq(other) {
      return Some(core::cmp::Ordering::Equal);
    }

    if other.overlaps(self) {
      return None;
    }

    if let Some(finish) = &self.finish {
      if let Some(other_start) = &other.start {
        if other_start >= finish {
          return Some(core::cmp::Ordering::Less);
        }
      }
    }

    if let Some(start) = &self.start {
      if let Some(other_finish) = &other.finish {
        if other_finish <= start {
          return Some(core::cmp::Ordering::Greater);
        }
      }
    }

    None
  }
}

impl<StartTz: TimeZone, FinishTz: TimeZone, Tz: TimeZone> PartialEq<DateTime<Tz>>
  for Timespan<StartTz, FinishTz>
{
  fn eq(&self, other: &DateTime<Tz>) -> bool {
    if let Some(start) = &self.start {
      if let Some(finish) = &self.finish {
        if start == &other.with_timezone(&start.timezone())
          && finish == &other.with_timezone(&finish.timezone())
        {
          return true;
        }
      }
    }

    false
  }
}

impl<StartTz: TimeZone, FinishTz: TimeZone, Tz: TimeZone> PartialOrd<DateTime<Tz>>
  for Timespan<StartTz, FinishTz>
{
  fn partial_cmp(&self, other: &DateTime<Tz>) -> Option<std::cmp::Ordering> {
    if self.contains(other) {
      return None;
    }

    if let Some(finish) = &self.finish {
      if other >= finish {
        return Some(core::cmp::Ordering::Less);
      }
    }

    if let Some(start) = &self.start {
      if other < start {
        return Some(core::cmp::Ordering::Greater);
      }
    }

    Some(core::cmp::Ordering::Equal)
  }
}
