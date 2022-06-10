pub mod blocks;
mod dig;
pub mod filters;
mod react_component_tag;
pub mod serialization;
pub mod tags;

use crate::{user_con_profiles, QueryData, SchemaData};
use async_graphql::Context;
use liquid::{partials::EagerCompiler, Error, Parser, ParserBuilder};

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

pub async fn build_liquid_parser(
  schema_data: &SchemaData,
  query_data: &QueryData,
) -> Result<Parser, liquid_core::Error> {
  let mut builder = ParserBuilder::with_stdlib()
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
    .block(blocks::SpoilerBlock);

  if let Some(cms_parent) = &query_data.cms_parent {
    builder = builder.partials(EagerCompiler::new(
      cms_parent
        .cms_partial_source(schema_data.db.clone())
        .await
        .map_err(|db_err| Error::with_msg(format!("Error loading partials: {}", db_err)))?,
    ));
  }

  builder.build()
}

pub async fn parse_and_render_in_graphql_context(
  ctx: &Context<'_>,
  content: &str,
) -> Result<String, async_graphql::Error> {
  let schema_data = ctx.data::<SchemaData>()?;
  let query_data = ctx.data::<QueryData>()?;

  let parser = build_liquid_parser(schema_data, query_data).await?;
  let template = parser.parse(content)?;

  let globals = liquid::object!({
    "num": 4f64,
    "timespan": liquid::object!({}),
    "convention": query_data.convention,
    "user_con_profile": std::option::Option::<user_con_profiles::Model>::None
  });

  let result = template.render(&globals);

  match result {
    Ok(content) => Ok(content),
    Err(error) => Err(async_graphql::Error::new(error.to_string())),
  }
}
