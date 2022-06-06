use std::io::Write;

use liquid::Error;
use liquid_core::{
  Expression, Language, ParseTag, Renderable, Result, Runtime, TagReflection, TagTokenIter, Value,
  ValueView,
};
use serde_json::json;

use super::write_react_component_tag;

#[derive(Copy, Clone, Debug, Default)]
pub struct AddToCalendarDropdownTag;

impl AddToCalendarDropdownTag {
  pub fn new() -> Self {
    Self::default()
  }
}

impl TagReflection for AddToCalendarDropdownTag {
  fn tag(&self) -> &'static str {
    "add_to_calendar_dropdown"
  }

  fn description(&self) -> &'static str {
    "Renders an \"Add to Calendar\" dropdown menu for a user to subscribe to their personal con \
      schedule.  The user's ical_secret must be provided.  The button CSS classes can be \
      customized."
  }
}

impl ParseTag for AddToCalendarDropdownTag {
  fn parse(
    &self,
    mut arguments: TagTokenIter<'_>,
    _options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    let ical_secret = arguments.expect_next("Identifier or literal expected.")?;
    let ical_secret = ical_secret.expect_value().into_result()?;

    let class_name = arguments
      .next()
      .and_then(|arg| arg.expect_value().into_result().ok())
      .unwrap_or(Expression::Literal(Value::scalar("btn btn-secondary")));

    arguments.expect_nothing()?;

    Ok(Box::new(AddToCalendarDropdown {
      ical_secret,
      class_name,
    }))
  }

  fn reflection(&self) -> &dyn TagReflection {
    self
  }
}

#[derive(Debug)]
struct AddToCalendarDropdown {
  ical_secret: Expression,
  class_name: Expression,
}

impl Renderable for AddToCalendarDropdown {
  fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
    let ical_secret = self.ical_secret.evaluate(runtime)?;
    if !ical_secret.is_scalar() {
      return Error::with_msg("ical_secret must be a string")
        .context(
          "add_to_calendar_dropdown",
          format!("{}", ical_secret.source()),
        )
        .into_err();
    }
    let ical_secret = ical_secret.to_kstr().into_owned();

    let class_name = self.class_name.evaluate(runtime)?;
    if !class_name.is_scalar() {
      return Error::with_msg("class_name must be a string")
        .context(
          "add_to_calendar_dropdown",
          format!("{}", class_name.source()),
        )
        .into_err();
    }
    let class_name = class_name.to_kstr().into_owned();

    write_react_component_tag(
      writer,
      "AddToCalendarDropdown",
      json!({
        "icalSecret": ical_secret,
          "className": class_name
      }),
    )
  }
}
