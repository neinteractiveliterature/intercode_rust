use std::io::Write;

use liquid::Error;
use liquid_core::{
  Expression, Language, ParseTag, Renderable, Result, Runtime, TagReflection, TagTokenIter,
  ValueView,
};
use serde_json::json;

use super::write_react_component_tag;

#[derive(Copy, Clone, Debug, Default)]
pub struct EventAdminMenuTag;

impl EventAdminMenuTag {
  pub fn new() -> Self {
    Self
  }
}

impl TagReflection for EventAdminMenuTag {
  fn tag(&self) -> &'static str {
    "event_admin_menu"
  }

  fn description(&self) -> &'static str {
    "Renders an event's admin menu, if the user is permitted to administer the event. \
      Requires specifying an event ID."
  }
}

impl ParseTag for EventAdminMenuTag {
  fn parse(
    &self,
    mut arguments: TagTokenIter<'_>,
    _options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    let event_id = arguments.expect_next("Identifier or literal expected.")?;
    let event_id = event_id.expect_value().into_result()?;

    arguments.expect_nothing()?;

    Ok(Box::new(EventAdminMenu { event_id }))
  }

  fn reflection(&self) -> &dyn TagReflection {
    self
  }
}

#[derive(Debug)]
struct EventAdminMenu {
  event_id: Expression,
}

impl Renderable for EventAdminMenu {
  fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
    let event_id = self.event_id.evaluate(runtime)?;
    if !event_id.is_scalar() {
      return Error::with_msg("event_id must be a string or number")
        .context("event_admin_menu", format!("{}", event_id.source()))
        .into_err();
    }
    let event_id = event_id.to_kstr().into_owned();

    write_react_component_tag(writer, "EventAdminMenu", json!({ "eventId": event_id }))
  }
}
