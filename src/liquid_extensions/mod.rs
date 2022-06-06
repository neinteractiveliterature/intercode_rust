pub mod blocks;
mod dig;
pub mod filters;
mod react_component_tag;
pub mod serialization;
pub mod tags;

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
    .tag(tags::AddToCalendarDropdownTag)
    .tag(tags::AssignGraphQLResultTag::new(
      query_data.cms_parent.clone(),
      query_data,
      schema_data,
    ))
    .tag(tags::CookieConsentTag)
    .tag(tags::EventAdminMenuTag)
    .tag(tags::EventRunsSectionTag)
    .tag(tags::FileUrlTag::new(
      query_data.cms_parent.clone(),
      schema_data.db.clone(),
    ))
    .tag(tags::LongFormEventDetailsTag)
    .tag(tags::MapTag)
    .tag(tags::MaximumEventSignupsPreviewTag {
      convention: query_data.convention.clone(),
    })
    .tag(tags::NewEventProposalButtonTag)
    .tag(tags::ShortFormEventDetailsTag)
    .tag(tags::WithdrawUserSignupButtonTag)
    .tag(tags::YouTubeTag)
    .block(blocks::SpoilerBlock)
    .build()
}
