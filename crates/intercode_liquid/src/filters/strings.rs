#![allow(clippy::box_default)]

use intercode_inflector::inflector::cases::titlecase::to_title_case;
use intercode_inflector::inflector::string::pluralize::to_plural;
use intercode_inflector::inflector::string::singularize::to_singular;
use liquid_core::Expression;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::{
  Display_filter, Filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
use liquid_core::{Value, ValueView};
use regex::Regex;

use intercode_inflector::IntercodeInflector;

use crate::invalid_argument;
use crate::invalid_input;

#[derive(Debug, FilterParameters)]
struct PluralizeArgs {
  #[parameter(
    description = "If input is a number, this will be the result if input is 1.",
    arg_type = "str"
  )]
  singular: Option<Expression>,
  #[parameter(
    description = "If input is a number, this will be the result if input is not 1.",
    arg_type = "str"
  )]
  plural: Option<Expression>,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
  name = "pluralize",
  description = "Can be used to either pluralize a singular noun, or to conditionally pluralize a noun based on a count.",
  parameters(PluralizeArgs),
  parsed(PluralizeFilter)
)]

pub struct Pluralize;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "pluralize"]
struct PluralizeFilter {
  #[parameters]
  args: PluralizeArgs,
}

impl Filter for PluralizeFilter {
  fn evaluate(&self, input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
    let args = self.args.evaluate(runtime)?;

    if input.is_nil() {
      return Ok(Value::Nil);
    }

    let input = input
      .as_scalar()
      .ok_or_else(|| invalid_input("String or number expected"))?;

    if let Some(count) = input.to_integer_strict() {
      let singular = args
        .singular
        .as_scalar()
        .ok_or_else(|| invalid_argument("singular", "String expected"))?;

      let plural = args
        .plural
        .as_scalar()
        .ok_or_else(|| invalid_argument("plural", "String expected"))?;

      if count == 1 {
        Ok(Value::scalar(format!(
          "{} {}",
          count,
          singular.to_kstr().into_string()
        )))
      } else {
        Ok(Value::scalar(format!(
          "{} {}",
          count,
          plural.to_kstr().into_string()
        )))
      }
    } else {
      Ok(Value::scalar(to_plural(input.to_kstr().as_str())))
    }
  }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
  name = "to_sentence",
  description = "Given an array of strings, outputs an English representation of that array.",
  parsed(ToSentenceFilter)
)]

pub struct ToSentence;

#[derive(Debug, Default, Display_filter)]
#[name = "to_sentence"]
struct ToSentenceFilter;

impl Filter for ToSentenceFilter {
  fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
    let input = input
      .as_array()
      .ok_or_else(|| invalid_input("Array of strings expected"))
      .unwrap();
    let count: usize = input.size().try_into().unwrap();

    match count {
      0 => Ok(Value::scalar("")),
      1 => Ok(Value::scalar(input.get(0).to_kstr().into_string())),
      2 => Ok(Value::scalar(format!(
        "{} and {}",
        input.get(0).to_kstr().into_string(),
        input.get(1).to_kstr().into_string()
      ))),
      _ => Ok(Value::scalar(input.values().enumerate().fold(
        "".to_string(),
        |acc, (index, item)| {
          if index == 0 {
            item.to_kstr().into_string()
          } else if index < count - 1 {
            format!("{}, {}", acc, item.to_kstr())
          } else {
            format!("{}, and {}", acc, item.to_kstr())
          }
        },
      ))),
    }
  }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
  name = "humanize",
  description = "Runs a string through the equivalent of Ruby on Rails's \"humanize\" filter.",
  parsed(HumanizeFilter)
)]

pub struct Humanize;

#[derive(Debug, Default, Display_filter)]
#[name = "humanize"]
struct HumanizeFilter;

impl Filter for HumanizeFilter {
  fn evaluate(&self, input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
    let inflector = runtime.registers().get_mut::<IntercodeInflector>();

    let input = input
      .as_scalar()
      .ok_or_else(|| invalid_input("String expected"))
      .unwrap()
      .to_kstr()
      .into_string();

    Ok(Value::scalar(inflector.humanize(&input)))
  }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
  name = "singularize",
  description = "Convert from a plural noun to a singular noun, if possible.",
  parsed(SingularizeFilter)
)]

pub struct Singularize;

#[derive(Debug, Default, Display_filter)]
#[name = "singularize"]
struct SingularizeFilter;

impl Filter for SingularizeFilter {
  fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
    let input = input
      .as_scalar()
      .ok_or_else(|| invalid_input("String expected"))
      .unwrap()
      .to_kstr()
      .into_string();

    Ok(Value::scalar(to_singular(&input)))
  }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
  name = "titleize",
  description = "Convert a string to Title Case.",
  parsed(TitleizeFilter)
)]

pub struct Titleize;

#[derive(Debug, Default, Display_filter)]
#[name = "titleize"]
struct TitleizeFilter;

impl Filter for TitleizeFilter {
  fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
    let input = input
      .as_scalar()
      .ok_or_else(|| invalid_input("String expected"))
      .unwrap()
      .to_kstr()
      .into_string();

    Ok(Value::scalar(to_title_case(&input)))
  }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
  name = "condense_whitespace",
  description = "Converts all whitespace in a string to single spaces, and strips whitespace off the \
    beginning and end.",
  parsed(CondenseWhitespaceFilter)
)]
pub struct CondenseWhitespace;

#[derive(Debug, Default, Display_filter)]
#[name = "condense_whitespace"]
struct CondenseWhitespaceFilter;

impl Filter for CondenseWhitespaceFilter {
  fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
    let input = input
      .as_scalar()
      .ok_or_else(|| invalid_input("String expected"))
      .unwrap()
      .to_kstr()
      .into_string();

    let whitespace_regex =
      Regex::new(r"\s+").expect("Could not parse whitespace regular expression");
    let result = whitespace_regex.replace_all(input.trim(), " ");
    Ok(Value::scalar(result.to_string()))
  }
}
