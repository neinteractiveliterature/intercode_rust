mod datetime;
mod digest;
mod links;
mod strings;

pub use datetime::*;
pub use digest::*;
pub use links::*;
use liquid::{Error, Parser, ParserBuilder};
pub use strings::*;

use crate::{QueryData, SchemaData};

fn invalid_input<S>(cause: S) -> Error
where
  S: Into<liquid_core::model::KString>,
{
  Error::with_msg("Invalid input").context("cause", cause)
}

fn invalid_argument<S>(argument: S, cause: S) -> Error
where
  S: Into<liquid_core::model::KString>,
{
  Error::with_msg("Invalid argument")
    .context("argument", argument)
    .context("cause", cause)
}

pub fn build_liquid_parser(
  schema_data: &SchemaData,
  query_data: &QueryData,
) -> Result<Parser, liquid_core::Error> {
  ParserBuilder::with_stdlib()
    .filter(Pluralize)
    .filter(EmailLink)
    .filter(ToSentence)
    .filter(Humanize)
    .filter(Singularize)
    .filter(Titleize)
    .filter(AbsoluteUrl {
      convention: query_data.convention.clone(),
    })
    .filter(CondenseWhitespace)
    .filter(MD5)
    .filter(DateWithLocalTime {
      convention: query_data.convention.clone(),
    })
    .filter(TimespanWithLocalTime {
      convention: query_data.convention.clone(),
      language_loader: schema_data.language_loader.clone(),
    })
    .build()
}
