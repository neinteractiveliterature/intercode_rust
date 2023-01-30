pub mod blocks;
pub mod cms_parent_partial_source;
mod dig;
pub mod filters;
mod markdown;
mod react_component_tag;
pub mod tags;

pub use markdown::*;
pub use react_component_tag::react_component_tag;
use seawater::ConnectionWrapper;

pub use dig::liquid_datetime_to_chrono_datetime;
use i18n_embed::fluent::FluentLanguageLoader;
use intercode_entities::{active_storage_blobs, cms_parent::CmsParent, conventions};
use liquid::{partials::PartialCompiler, Error, Parser, ParserBuilder};
use std::{fmt::Debug, future::Future, pin::Pin, sync::Weak};
use tags::GraphQLExecutorBuilder;

pub trait GraphQLExecutor: Debug + Send + Sync {
  fn execute(
    &self,
    request: async_graphql::Request,
  ) -> Pin<Box<dyn Future<Output = async_graphql::Response> + Send + '_>>;
}

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
  convention: Option<&conventions::Model>,
  language_loader: Weak<FluentLanguageLoader>,
  cms_parent: &CmsParent,
  db: ConnectionWrapper,
  user_signed_in: bool,
  graphql_executor_builder: Box<dyn GraphQLExecutorBuilder>,
  partial_compiler: impl PartialCompiler,
) -> Result<Parser, liquid_core::Error> {
  let builder = ParserBuilder::with_stdlib()
    .filter(filters::Pluralize)
    .filter(filters::EmailLink::new(user_signed_in))
    .filter(filters::ToSentence)
    .filter(filters::Humanize)
    .filter(filters::Singularize)
    .filter(filters::Titleize)
    .filter(filters::AbsoluteUrl {
      convention_domain: convention.map(|c| c.domain.clone()),
    })
    .filter(filters::CondenseWhitespace)
    .filter(filters::MD5)
    .filter(filters::DateWithLocalTime {
      convention_timezone: convention.and_then(|c| c.timezone_name.clone()),
    })
    .filter(filters::TimespanWithLocalTime {
      convention_timezone: convention.and_then(|c| c.timezone_name.clone()),
      language_loader,
    })
    .tag(tags::AddToCalendarDropdownTag)
    .tag(tags::AssignGraphQLResultTag::new(
      cms_parent,
      db.clone(),
      graphql_executor_builder,
    ))
    .tag(tags::CookieConsentTag)
    .tag(tags::EventAdminMenuTag)
    .tag(tags::EventRunsSectionTag)
    .tag(tags::FileUrlTag::new(cms_parent, db))
    .tag(tags::LongFormEventDetailsTag)
    .tag(tags::MapTag)
    .tag(tags::MaximumEventSignupsPreviewTag {
      convention_timezone: convention.and_then(|c| c.timezone_name.clone()),
    })
    .tag(tags::NewEventProposalButtonTag)
    .tag(tags::PageUrlTag)
    .tag(tags::ShortFormEventDetailsTag)
    .tag(tags::WithdrawUserSignupButtonTag)
    .tag(tags::YouTubeTag)
    .block(blocks::SpoilerBlock)
    .partials(partial_compiler);

  builder.build()
}

pub fn build_active_storage_blob_url(blob: &active_storage_blobs::Model) -> String {
  // TODO do something actually real here
  format!("https://assets.neilhosting.net/{}", blob.key)
}
