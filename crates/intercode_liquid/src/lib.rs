pub mod blocks;
pub mod cms_parent_partial_source;
mod dig;
pub mod drops;
pub mod filters;
mod react_component_tag;
pub mod tags;

pub use react_component_tag::react_component_tag;

use std::{fmt::Debug, future::Future, pin::Pin, sync::Arc};

use i18n_embed::fluent::FluentLanguageLoader;
use intercode_entities::{cms_parent::CmsParent, conventions};
use liquid::{partials::PartialCompiler, Error, Parser, ParserBuilder};
use sea_orm::DatabaseConnection;

pub trait GraphQLExecutor: Debug + Clone + Send + Sync {
  fn execute(
    &self,
    request: impl Into<async_graphql::Request>,
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

pub fn build_liquid_parser<'a>(
  convention: &'a Arc<Option<conventions::Model>>,
  language_loader: &'a Arc<FluentLanguageLoader>,
  cms_parent: &'a Arc<CmsParent>,
  db: &'a Arc<DatabaseConnection>,
  user_signed_in: bool,
  graphql_executor: impl GraphQLExecutor + 'static,
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
      convention: convention.clone(),
    })
    .filter(filters::CondenseWhitespace)
    .filter(filters::MD5)
    .filter(filters::DateWithLocalTime {
      convention: convention.clone(),
    })
    .filter(filters::TimespanWithLocalTime {
      convention: convention.clone(),
      language_loader: language_loader.clone(),
    })
    .tag(tags::AddToCalendarDropdownTag)
    .tag(tags::AssignGraphQLResultTag::new(
      cms_parent.clone(),
      db.clone(),
      graphql_executor,
    ))
    .tag(tags::CookieConsentTag)
    .tag(tags::EventAdminMenuTag)
    .tag(tags::EventRunsSectionTag)
    .tag(tags::FileUrlTag::new(cms_parent.clone(), db.clone()))
    .tag(tags::LongFormEventDetailsTag)
    .tag(tags::MapTag)
    .tag(tags::MaximumEventSignupsPreviewTag {
      convention: convention.clone(),
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
