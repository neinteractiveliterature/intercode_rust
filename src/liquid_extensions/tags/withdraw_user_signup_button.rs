use std::io::Write;

use liquid::Error;
use liquid_core::{
  Expression, Language, ParseTag, Renderable, Result, Runtime, TagReflection, TagTokenIter, Value,
  ValueView,
};
use serde_json::json;

use super::super::dig::dig_value;
use super::write_react_component_tag;

#[derive(Copy, Clone, Debug, Default)]
pub struct WithdrawUserSignupButtonTag;

impl WithdrawUserSignupButtonTag {
  pub fn new() -> Self {
    Self::default()
  }
}

impl TagReflection for WithdrawUserSignupButtonTag {
  fn tag(&self) -> &'static str {
    "withdraw_user_signup_button"
  }

  fn description(&self) -> &'static str {
    "Renders a \"Withdraw\" button for an existing signup.  The button text and the button
      CSS classes can be customized."
  }
}

impl ParseTag for WithdrawUserSignupButtonTag {
  fn parse(
    &self,
    mut arguments: TagTokenIter<'_>,
    _options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    let signup = arguments
      .expect_next("withdraw_user_signup_button requires a signup object")?
      .expect_value()
      .into_result()?;

    let button_text = arguments
      .next()
      .and_then(|arg| arg.expect_value().into_result().ok())
      .unwrap_or(Expression::Literal(Value::Nil));

    let button_class = arguments
      .next()
      .and_then(|arg| arg.expect_value().into_result().ok())
      .unwrap_or(Expression::Literal(Value::Nil));

    arguments.expect_nothing()?;

    Ok(Box::new(WithdrawUserSignupButton {
      signup,
      button_text,
      button_class,
    }))
  }

  fn reflection(&self) -> &dyn TagReflection {
    self
  }
}

#[derive(Debug)]
struct WithdrawUserSignupButton {
  signup: Expression,
  button_text: Expression,
  button_class: Expression,
}

impl Renderable for WithdrawUserSignupButton {
  fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
    let signup = self.signup.evaluate(runtime)?;
    let signup_source = format!("{}", signup.source());
    let signup = signup.as_object().ok_or(
      Error::with_msg("signup must be an object")
        .context("withdraw_user_signup_button", &signup_source),
    )?;

    let event_title = dig_value(
      signup,
      vec!["event", "title"],
      "withdraw_user_signup_button",
      &signup_source,
    )?;
    let run_id = dig_value(
      signup,
      vec!["run", "id"],
      "withdraw_user_signup_button",
      &signup_source,
    )?;

    let button_text = self.button_text.evaluate(runtime)?;
    if !button_text.is_scalar() {
      return Error::with_msg("button_text must be a string")
        .context(
          "withdraw_user_signup_button",
          format!("{}", button_text.source()),
        )
        .into_err();
    }
    let button_text = button_text.to_kstr().into_owned();

    let button_class = self.button_class.evaluate(runtime)?;
    if !button_class.is_scalar() {
      return Error::with_msg("button_class must be a string")
        .context(
          "withdraw_user_signup_button",
          format!("{}", button_class.source()),
        )
        .into_err();
    }
    let button_class = button_class.to_kstr().into_owned();

    write_react_component_tag(
      writer,
      "WithdrawMySignupButton",
      json!({
        "caption": button_text,
          "className": button_class,
          "reloadOnSuccess": true,
          "event": json!({
            "title": event_title.to_kstr()
          }),
          "run": json!({
            "id": run_id.to_kstr()
          })
      }),
    )
  }
}
