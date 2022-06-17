use std::sync::Arc;

use crate::invalid_input;
use intercode_entities::conventions;
use liquid_core::{
  Display_filter, Filter, FilterReflection, ParseFilter, Result, Runtime, Value, ValueView,
};
use url::Url;

#[derive(Clone, FilterReflection)]
#[filter(
  name = "email_link",
  description = "Outputs either a clickable mailto: link (if the user is currently logged in), or an obfuscated \
    email (if the user is not logged in).  The intent of this is to be a spammer-safe way to link to email addresses.",
  parsed(EmailLinkFilter)
)]

pub struct EmailLink {
  user_signed_in: bool,
}

impl EmailLink {
  pub fn new(user_signed_in: bool) -> Self {
    EmailLink { user_signed_in }
  }
}

impl ParseFilter for EmailLink {
  fn parse(&self, _arguments: liquid_core::parser::FilterArguments) -> Result<Box<dyn Filter>> {
    Ok(Box::new(EmailLinkFilter {
      user_signed_in: self.user_signed_in,
    }))
  }

  fn reflection(&self) -> &dyn FilterReflection {
    self
  }
}

#[derive(Debug, Default, Display_filter)]
#[name = "email_link"]
struct EmailLinkFilter {
  user_signed_in: bool,
}

impl Filter for EmailLinkFilter {
  fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
    let input = input.to_value();

    match input {
      Value::Nil => Ok(Value::scalar("")),
      Value::Scalar(email) => {
        let email = email.to_kstr().into_string();

        if self.user_signed_in {
          Ok(Value::scalar(format!(
            "<a href=\"mailto:{}\">{}</a>",
            email, email
          )))
        } else {
          Ok(Value::scalar(
            email.replace('@', " AT ").replace('.', " DOT "),
          ))
        }
      }
      _ => Err(invalid_input("String expected")),
    }
  }
}

#[derive(Clone, FilterReflection)]
#[filter(
  name = "absolute_url",
  description = "Given a relative URL, turns it into an absolute URL for the current convention.  Given an absolute \
    URL, changes the hostname to the current convention host."
)]
pub struct AbsoluteUrl {
  pub convention: Arc<Option<conventions::Model>>,
}

impl ParseFilter for AbsoluteUrl {
  fn parse(
    &self,
    _arguments: liquid_core::parser::FilterArguments,
  ) -> Result<Box<dyn liquid_core::Filter>> {
    Ok(Box::new(AbsoluteUrlFilter {
      convention: self.convention.clone(),
    }))
  }

  fn reflection(&self) -> &dyn FilterReflection {
    self
  }
}

#[derive(Debug, Default, Display_filter)]
#[name = "absolute_url"]
struct AbsoluteUrlFilter {
  convention: Arc<Option<conventions::Model>>,
}

impl Filter for AbsoluteUrlFilter {
  fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
    let input = input.to_value();
    let convention = &self.convention;

    match input {
      Value::Nil => Ok(Value::scalar("")),
      Value::Scalar(url) => match convention.as_ref() {
        None => Ok(Value::Scalar(url)),
        Some(convention) => {
          let options = Url::options();
          let convention_base = Url::parse(&format!("http://{}", convention.domain)).ok();
          let base_url = options.base_url(convention_base.as_ref());
          let url = base_url.parse(&url.to_kstr().into_string());
          match url {
            Ok(mut parsed_url) => {
              let set_host_result = parsed_url.set_host(Some(&convention.domain));
              match set_host_result {
                Ok(_) => Ok(Value::scalar(parsed_url.to_string())),
                Err(error) => Err(invalid_input(format!("Can't set host on URL: {}", error))),
              }
            }
            Err(error) => Err(invalid_input(format!("Can't parse URL: {}", error))),
          }
        }
      },
      _ => Err(invalid_input("String expected")),
    }
  }
}
