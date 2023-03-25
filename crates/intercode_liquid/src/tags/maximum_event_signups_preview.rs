use super::write_react_component_tag;
use crate::dig::{get_array_from_value, get_datetime_from_value, get_scalar_from_value};
use crate::invalid_argument;
use liquid::Error;
use liquid_core::{
  Expression, Language, ParseTag, Renderable, Result, Runtime, TagReflection, TagTokenIter,
  ValueView,
};
use serde_json::json;
use std::io::Write;

#[derive(Clone, Debug, Default)]
pub struct MaximumEventSignupsPreviewTag {
  pub convention_timezone: Option<String>,
}

impl MaximumEventSignupsPreviewTag {
  pub fn new() -> Self {
    Self::default()
  }
}

impl TagReflection for MaximumEventSignupsPreviewTag {
  fn tag(&self) -> &'static str {
    "maximum_event_signups_preview"
  }

  fn description(&self) -> &'static str {
    "Renders a calendar showing the maximum event signup schedule for a convention."
  }
}

impl ParseTag for MaximumEventSignupsPreviewTag {
  fn parse(
    &self,
    mut arguments: TagTokenIter<'_>,
    _options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    let scheduled_value = arguments
      .expect_next("maximum_event_signups_preview requires a scheduled_value object")?
      .expect_value()
      .into_result()?;

    arguments.expect_nothing()?;

    Ok(Box::new(MaximumEventSignupsPreview {
      convention_timezone: self.convention_timezone.clone(),
      scheduled_value,
    }))
  }

  fn reflection(&self) -> &dyn TagReflection {
    self
  }
}

#[derive(Debug)]
struct MaximumEventSignupsPreview {
  scheduled_value: Expression,
  convention_timezone: Option<String>,
}

impl Renderable for MaximumEventSignupsPreview {
  fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
    let scheduled_value = self.scheduled_value.evaluate(runtime)?;
    let scheduled_value_source = format!("{}", scheduled_value.source());
    let scheduled_value = scheduled_value.as_object().ok_or_else(|| {
      Error::with_msg("scheduled_value must be an object")
        .context("maximum_event_signups_preview", &scheduled_value_source)
    })?;

    let timespans = get_array_from_value(
      scheduled_value,
      "timespans",
      "maximum_event_signups_preview",
      &scheduled_value_source,
    )?;

    let timespans = timespans
      .values()
      .map(|timespan| {
        let timespan = timespan.as_object().ok_or_else(|| {
          invalid_argument(
            timespans.source().to_string(),
            String::from("timespans must be an array of objects"),
          )
        })?;
        let start = get_datetime_from_value(
          &timespan,
          "start",
          "maximum_event_signups_preview",
          &timespan.source().to_string(),
        )?;
        let finish = get_datetime_from_value(
          &timespan,
          "finish",
          "maximum_event_signups_preview",
          &timespan.source().to_string(),
        )?;
        let value = get_scalar_from_value(
          &timespan,
          "value",
          "maximum_event_signups_preview",
          &timespan.source().to_string(),
        )
        .ok();

        Ok(json!({ "start": start.to_rfc3339(), "finish": finish.to_rfc3339(), "value": value }))
      })
      .collect::<Result<Vec<serde_json::Value>, Error>>()?;

    let timezone_name = self.convention_timezone.as_deref().unwrap_or_default();

    write_react_component_tag(
      writer,
      "MaximumEventSignupsPreview",
      json!({ "timespans": timespans, "timezone_name": timezone_name }),
    )
  }
}
