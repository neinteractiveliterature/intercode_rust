use std::sync::Arc;

use chrono::TimeZone;
use chrono_tz::Tz;
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;
use liquid_core::{
  Display_filter, Expression, Filter, FilterParameters, FilterReflection, FromFilterParameters,
  ParseFilter, Result, Runtime, Value, ValueView,
};

use crate::{conventions, timespan::Timespan};

use crate::liquid_extensions::invalid_input;

fn find_effective_timezone(
  timezone_name: Option<&String>,
  convention: Option<&conventions::Model>,
) -> Option<Tz> {
  timezone_name
    .or_else(|| convention.and_then(|convention| convention.timezone_name.as_ref()))
    .and_then(|timezone_name| timezone_name.parse::<Tz>().ok())
}

fn liquid_datetime_to_chrono_datetime(
  input: &liquid_core::model::DateTime,
) -> chrono::DateTime<chrono::FixedOffset> {
  let offset = chrono::FixedOffset::east(input.offset().whole_seconds());
  offset
    .ymd(input.year(), input.month().into(), input.day().into())
    .and_hms_nano(
      input.hour().into(),
      input.minute().into(),
      input.second().into(),
      input.nanosecond().into(),
    )
}

fn common_prefix(a: &str, b: &str, delimiter: &str) -> String {
  a.split(delimiter)
    .zip(b.split(delimiter))
    .take_while(|(a_part, b_part)| a_part == b_part)
    .map(|(a_part, _b_part)| a_part)
    .collect::<Vec<&str>>()
    .join(delimiter)
}

fn common_suffix(a: &str, b: &str, delimiter: &str) -> String {
  common_prefix(
    &a.chars().rev().collect::<String>(),
    &b.chars().rev().collect::<String>(),
    delimiter,
  )
  .chars()
  .rev()
  .collect()
}

fn remove_common_middle(a: &str, b: &str, delimiter: &str) -> (String, String) {
  let prefix = common_prefix(a, b, delimiter);
  let suffix = common_suffix(a, b, delimiter);

  (
    a.strip_suffix(&suffix).unwrap_or(a).to_string(),
    b.strip_prefix(&prefix).unwrap_or(b).to_string(),
  )
}

#[derive(Debug, FilterParameters)]
struct DateWithLocalTimeArgs {
  #[parameter(
    description = "A time formatting string, like the one the built-in Liquid \"date\" \
      filter uses (see http://strftime.net for examples).  We recommend \
      including \"%Z\" in this string in order to have an explicit time zone \
      specifier.",
    arg_type = "str"
  )]
  format: Expression,
  #[parameter(
    description = "An IANA timezone name to use for the default format.  If \
      not given, this filter will try to use the convention's \
      local timezone (if one exists).",
    arg_type = "str"
  )]
  timezone_name: Option<Expression>,
}

#[derive(Clone, FilterReflection)]
#[filter(
  name = "date_with_local_time",
  description = "Given a time object, format it in the given timezone, translating to the user's local \
    time if it isn't the same.",
  parameters(DateWithLocalTimeArgs),
  parsed(DateWithLocalTimeFilter)
)]
pub struct DateWithLocalTime {
  pub convention: Option<conventions::Model>,
}

impl ParseFilter for DateWithLocalTime {
  fn parse(&self, arguments: liquid_core::parser::FilterArguments) -> Result<Box<dyn Filter>> {
    let args = DateWithLocalTimeArgs::from_args(arguments)?;
    Ok(Box::new(DateWithLocalTimeFilter {
      args,
      convention: self.convention.clone(),
    }))
  }

  fn reflection(&self) -> &dyn FilterReflection {
    self
  }
}

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "date_with_local_time"]
struct DateWithLocalTimeFilter {
  convention: Option<conventions::Model>,
  #[parameters]
  args: DateWithLocalTimeArgs,
}

impl Filter for DateWithLocalTimeFilter {
  fn evaluate(&self, input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
    let input = input
      .as_scalar()
      .ok_or_else(|| invalid_input("String or DateTime expected"))?
      .to_date_time()
      .ok_or_else(|| invalid_input("Cannot parse input as datetime"))?;
    let args = self.args.evaluate(runtime)?;
    let format_str = args.format.to_string();
    let format = chrono::format::strftime::StrftimeItems::new(&format_str);

    let datetime = liquid_datetime_to_chrono_datetime(&input);
    let timezone_name = args
      .timezone_name
      .as_ref()
      .and_then(|expr| Some(expr.to_string()));

    let tz = find_effective_timezone(timezone_name.as_ref(), self.convention.as_ref());

    if let Some(tz) = tz {
      let datetime_in_tz = datetime.with_timezone(&tz);
      Ok(Value::scalar(
        datetime_in_tz.format_with_items(format).to_string(),
      ))
    } else {
      Ok(Value::scalar(
        datetime.format_with_items(format).to_string(),
      ))
    }
  }
}

#[derive(Debug, FilterParameters)]
struct TimespanWithLocalTimeArgs {
  #[parameter(
    description = "A time formatting string, like the one the built-in Liquid \"date\" \
      filter uses (see http://strftime.net for examples).  We recommend \
      including \"%Z\" in this string in order to have an explicit time zone \
      specifier.",
    arg_type = "str"
  )]
  format: Expression,
  #[parameter(
    description = "An IANA timezone name to use for the default format.  If \
      not given, this filter will try to use the convention's \
      local timezone (if one exists).",
    arg_type = "str"
  )]
  timezone_name: Option<Expression>,
}

#[derive(Clone, FilterReflection)]
#[filter(
  name = "timespan_with_local_time",
  description = "Given a timespan, format it in the given timezone, translating to the user's local \
    time if it isn't the same.  Automatically removes duplicate verbiage in the middle (e.g. \
    day of week, time zone, etc.)",
  parameters(TimespanWithLocalTimeArgs),
  parsed(TimespanWithLocalTimeFilter)
)]
pub struct TimespanWithLocalTime {
  pub convention: Option<conventions::Model>,
  pub language_loader: Arc<FluentLanguageLoader>,
}

impl ParseFilter for TimespanWithLocalTime {
  fn parse(&self, arguments: liquid_core::parser::FilterArguments) -> Result<Box<dyn Filter>> {
    let args = TimespanWithLocalTimeArgs::from_args(arguments)?;
    Ok(Box::new(TimespanWithLocalTimeFilter {
      args,
      convention: self.convention.clone(),
      language_loader: self.language_loader.clone(),
    }))
  }

  fn reflection(&self) -> &dyn FilterReflection {
    self
  }
}

#[derive(Debug, Display_filter)]
#[name = "timespan_with_local_time"]
struct TimespanWithLocalTimeFilter {
  convention: Option<conventions::Model>,
  language_loader: Arc<FluentLanguageLoader>,

  #[parameters]
  args: TimespanWithLocalTimeArgs,
}

impl Filter for TimespanWithLocalTimeFilter {
  fn evaluate(&self, input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
    let input = input
      .as_object()
      .ok_or_else(|| invalid_input("Timespan expected"))?;
    let start = input
      .get("start")
      .as_scalar()
      .map(|start| {
        start
          .to_date_time()
          .map(|dt| Some(liquid_datetime_to_chrono_datetime(&dt)))
          .ok_or_else(|| invalid_input("Cannot parse start as datetime"))
      })
      .unwrap_or(Ok(None))?;
    let finish = input
      .get("finish")
      .as_scalar()
      .map(|finish| {
        finish
          .to_date_time()
          .map(|dt| Some(liquid_datetime_to_chrono_datetime(&dt)))
          .ok_or_else(|| invalid_input("Cannot parse finish as datetime"))
      })
      .unwrap_or(Ok(None))?;
    let parsed_timespan = Timespan { start, finish };

    if parsed_timespan.start == None && parsed_timespan.finish == None {
      return Ok(Value::scalar(fl!(
        self.language_loader,
        "start_and_finish_unbounded"
      )));
    }

    let args = self.args.evaluate(runtime)?;
    let format_str = args.format.to_string();
    let timezone_name = args
      .timezone_name
      .as_ref()
      .and_then(|expr| Some(expr.to_string()));
    let tz = find_effective_timezone(timezone_name.as_ref(), self.convention.as_ref());

    let start_desc: String;
    let finish_desc: String;
    if let Some(tz) = tz {
      let converted_timespan = parsed_timespan.with_timezone(&tz);
      start_desc = if let Some(start) = converted_timespan.start {
        start.format(&format_str).to_string()
      } else {
        fl!(self.language_loader, "start_unbounded")
      };
      finish_desc = if let Some(finish) = converted_timespan.finish {
        finish.format(&format_str).to_string()
      } else {
        fl!(self.language_loader, "finish_unbounded")
      };
    } else {
      start_desc = if let Some(start) = parsed_timespan.start {
        start.format(&format_str).to_string()
      } else {
        fl!(self.language_loader, "start_unbounded")
      };
      finish_desc = if let Some(finish) = parsed_timespan.finish {
        finish.format(&format_str).to_string()
      } else {
        fl!(self.language_loader, "finish_unbounded")
      };
    }

    if parsed_timespan.finish == None {
      Ok(Value::scalar(fl!(
        self.language_loader,
        "timespan_with_unbounded_finish",
        start = start_desc
      )))
    } else {
      let (deduped_start, deduped_finish) = remove_common_middle(&start_desc, &finish_desc, " ");

      if deduped_start.trim().len() == 0 || deduped_finish.trim().len() == 0 {
        Ok(Value::scalar(fl!(
          self.language_loader,
          "timespan_with_bounded_finish",
          start = start_desc,
          finish = finish_desc
        )))
      } else {
        Ok(Value::scalar(fl!(
          self.language_loader,
          "timespan_with_bounded_finish",
          start = deduped_start,
          finish = deduped_finish
        )))
      }
    }
  }
}
