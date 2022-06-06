use std::io::Write;

use liquid::Error;
use liquid_core::{
  Expression, Language, ParseTag, Renderable, Result, Runtime, TagReflection, TagTokenIter,
  ValueView,
};
use serde_json::json;

use super::write_react_component_tag;

#[derive(Copy, Clone, Debug, Default)]
pub struct CookieConsentTag;

impl CookieConsentTag {
  pub fn new() -> Self {
    Self::default()
  }
}

impl TagReflection for CookieConsentTag {
  fn tag(&self) -> &'static str {
    "cookie_consent"
  }

  fn description(&self) -> &'static str {
    "Renders a cookie consent bar.  Requires a URL for a cookie policy to link to."
  }
}

impl ParseTag for CookieConsentTag {
  fn parse(
    &self,
    mut arguments: TagTokenIter<'_>,
    _options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    let cookie_policy_url = arguments.expect_next("Identifier or literal expected.")?;
    let cookie_policy_url = cookie_policy_url.expect_value().into_result()?;

    arguments.expect_nothing()?;

    Ok(Box::new(CookieConsent { cookie_policy_url }))
  }

  fn reflection(&self) -> &dyn TagReflection {
    self
  }
}

#[derive(Debug)]
struct CookieConsent {
  cookie_policy_url: Expression,
}

impl Renderable for CookieConsent {
  fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
    let cookie_policy_url = self.cookie_policy_url.evaluate(runtime)?;
    if !cookie_policy_url.is_scalar() {
      return Error::with_msg("cookie_policy_url must be a string")
        .context("cookie_consent", format!("{}", cookie_policy_url.source()))
        .into_err();
    }
    let cookie_policy_url = cookie_policy_url.to_kstr().into_owned();

    write_react_component_tag(
      writer,
      "CookieConsent",
      json!({ "cookiePolicyUrl": cookie_policy_url }),
    )
  }
}
