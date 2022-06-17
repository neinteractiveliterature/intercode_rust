use std::io::Write;

use liquid::Error;
use liquid_core::{
  Expression, Language, ParseTag, Renderable, Result, Runtime, TagReflection, TagTokenIter,
  ValueView,
};
use serde_json::json;

use super::write_react_component_tag;

#[derive(Copy, Clone, Debug, Default)]
pub struct ShortFormEventDetailsTag;

impl ShortFormEventDetailsTag {
  pub fn new() -> Self {
    Self::default()
  }
}

impl TagReflection for ShortFormEventDetailsTag {
  fn tag(&self) -> &'static str {
    "short_form_event_details"
  }

  fn description(&self) -> &'static str {
    "Renders the short-form event details list (usually including stuff like the event's team \
      members, contact email, authors, etc.)  Requires specifying an event ID."
  }
}

impl ParseTag for ShortFormEventDetailsTag {
  fn parse(
    &self,
    mut arguments: TagTokenIter<'_>,
    _options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    let event_id = arguments.expect_next("Identifier or literal expected.")?;
    let event_id = event_id.expect_value().into_result()?;

    arguments.expect_nothing()?;

    Ok(Box::new(ShortFormEventDetails { event_id }))
  }

  fn reflection(&self) -> &dyn TagReflection {
    self
  }
}

#[derive(Debug)]
struct ShortFormEventDetails {
  event_id: Expression,
}

impl Renderable for ShortFormEventDetails {
  fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
    let event_id = self.event_id.evaluate(runtime)?;
    if !event_id.is_scalar() {
      return Error::with_msg("event_id must be a string or number")
        .context("short_form_event_details", format!("{}", event_id.source()))
        .into_err();
    }
    let event_id = event_id.to_kstr().into_owned();

    write_react_component_tag(
      writer,
      "ShortFormEventDetails",
      json!({ "eventId": event_id }),
    )
  }
}
