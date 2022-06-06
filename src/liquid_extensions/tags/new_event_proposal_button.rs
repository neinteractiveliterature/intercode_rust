use std::io::Write;

use liquid::Error;
use liquid_core::{
  Expression, Language, ParseTag, Renderable, Result, Runtime, TagReflection, TagTokenIter, Value,
  ValueView,
};
use serde_json::json;

use super::write_react_component_tag;

#[derive(Copy, Clone, Debug, Default)]
pub struct NewEventProposalButtonTag;

impl NewEventProposalButtonTag {
  pub fn new() -> Self {
    Self::default()
  }
}

impl TagReflection for NewEventProposalButtonTag {
  fn tag(&self) -> &'static str {
    "new_event_proposal_button"
  }

  fn description(&self) -> &'static str {
    "Renders a \"Propose an event\" button.  This will automatically render as a \
      \"Log in to propose\" button if the user is not logged in.  The button text and the button \
      CSS classes can be customized."
  }
}

impl ParseTag for NewEventProposalButtonTag {
  fn parse(
    &self,
    mut arguments: TagTokenIter<'_>,
    _options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    let button_text = arguments
      .next()
      .and_then(|arg| arg.expect_value().into_result().ok())
      .unwrap_or(Expression::Literal(Value::scalar("Propose an event")));

    let button_class = arguments
      .next()
      .and_then(|arg| arg.expect_value().into_result().ok())
      .unwrap_or(Expression::Literal(Value::scalar("btn btn-secondary")));

    arguments.expect_nothing()?;

    Ok(Box::new(NewEventProposalButton {
      button_text,
      button_class,
    }))
  }

  fn reflection(&self) -> &dyn TagReflection {
    self
  }
}

#[derive(Debug)]
struct NewEventProposalButton {
  button_text: Expression,
  button_class: Expression,
}

impl Renderable for NewEventProposalButton {
  fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
    let button_text = self.button_text.evaluate(runtime)?;
    if !button_text.is_scalar() {
      return Error::with_msg("button_text must be a string")
        .context(
          "new_event_proposal_button",
          format!("{}", button_text.source()),
        )
        .into_err();
    }
    let button_text = button_text.to_kstr().into_owned();

    let button_class = self.button_class.evaluate(runtime)?;
    if !button_class.is_scalar() {
      return Error::with_msg("button_class must be a string")
        .context(
          "new_event_proposal_button",
          format!("{}", button_class.source()),
        )
        .into_err();
    }
    let button_class = button_class.to_kstr().into_owned();

    write_react_component_tag(
      writer,
      "NewEventProposalButton",
      json!({
        "caption": button_text,
          "className": button_class
      }),
    )
  }
}
