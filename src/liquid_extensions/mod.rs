pub mod filters;

use crate::{QueryData, SchemaData};
use liquid::{Error, Parser, ParserBuilder};

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
    .filter(filters::Pluralize)
    .filter(filters::EmailLink)
    .filter(filters::ToSentence)
    .filter(filters::Humanize)
    .filter(filters::Singularize)
    .filter(filters::Titleize)
    .filter(filters::AbsoluteUrl {
      convention: query_data.convention.clone(),
    })
    .filter(filters::CondenseWhitespace)
    .filter(filters::MD5)
    .filter(filters::DateWithLocalTime {
      convention: query_data.convention.clone(),
    })
    .filter(filters::TimespanWithLocalTime {
      convention: query_data.convention.clone(),
      language_loader: schema_data.language_loader.clone(),
    })
    .build()
}
