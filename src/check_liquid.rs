use std::{collections::HashMap, fmt::Display, sync::Arc};

use crate::{
  connect_database, liquid_renderer::IntercodeLiquidRenderer, server::build_language_loader,
  setup_tracing,
};
use async_graphql::Result;
use chrono_tz::UTC;
use futures::{FutureExt, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use intercode_entities::{
  cms_layouts, cms_parent::CmsParent, conventions, pages, root_sites, user_con_profiles, users,
  UserNames,
};
use intercode_graphql::{
  cms_rendering_context::CmsRenderingContext, ArcQueryData, OwnedQueryData, QueryData, SchemaData,
};
use intercode_liquid::cms_parent_partial_source::PreloadPartialsStrategy;
use intercode_policies::AuthorizationInfo;
use itertools::Itertools;
use liquid::object;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use seawater::ConnectionWrapper;
use tracing_subscriber::EnvFilter;

enum ResourceDescriptor {
  Page(Option<String>),
  Layout(Option<String>),
}

impl Display for ResourceDescriptor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ResourceDescriptor::Page(name) => f.write_fmt(format_args!(
        "page {}",
        name.as_deref().unwrap_or("<untitled>")
      )),
      ResourceDescriptor::Layout(name) => f.write_fmt(format_args!(
        "layout {}",
        name.as_deref().unwrap_or("<untitled>")
      )),
    }
  }
}

struct LiquidRenderingError {
  convention_name: Option<String>,
  resource_descriptor: ResourceDescriptor,
  user_descriptor: String,
  error: async_graphql::Error,
}

impl Display for LiquidRenderingError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let convention_descriptor = self.convention_name.as_deref().unwrap_or("root site");
    f.write_fmt(format_args!(
      "Error rendering {} in {}: {:?}",
      self.resource_descriptor, convention_descriptor, self.error
    ))
  }
}

async fn find_or_build_user_con_profile(
  user: Option<&users::Model>,
  convention: Option<&conventions::Model>,
  connection_wrapper: &ConnectionWrapper,
) -> Option<user_con_profiles::Model> {
  match (user, convention) {
    (Some(user), Some(convention)) => user_con_profiles::Entity::find()
      .filter(user_con_profiles::Column::ConventionId.eq(convention.id))
      .filter(user_con_profiles::Column::UserId.eq(user.id))
      .one(connection_wrapper)
      .await
      .ok()
      .unwrap_or_else(|| {
        Some(user_con_profiles::Model {
          user_id: user.id,
          convention_id: convention.id,
          ..Default::default()
        })
      }),
    _ => None,
  }
}

fn build_query_data(
  cms_parent: CmsParent,
  current_user: Option<users::Model>,
  parent_convention: Option<conventions::Model>,
  connection_wrapper: &ConnectionWrapper,
  user_con_profile: Option<user_con_profiles::Model>,
) -> QueryData {
  let query_data = OwnedQueryData::new(
    cms_parent,
    current_user,
    parent_convention.clone(),
    connection_wrapper.clone(),
    parent_convention
      .and_then(|c| c.timezone_name)
      .and_then(|tz_name| tz_name.parse().ok())
      .unwrap_or(UTC),
    user_con_profile,
  );

  Box::new(ArcQueryData::new(query_data))
}

async fn render_page(
  page: pages::Model,
  convention: Option<conventions::Model>,
  user: Option<users::Model>,
  root_site: &root_sites::Model,
  connection_wrapper: &ConnectionWrapper,
  schema_data: &SchemaData,
) -> Result<String, LiquidRenderingError> {
  let parent_convention = match page.parent_type.as_deref() {
    Some("Convention") => convention,
    _ => None,
  };
  let (query_data, renderer) = build_query_data_and_renderer(
    parent_convention,
    root_site,
    user,
    connection_wrapper,
    schema_data,
  )
  .await;
  let rendering_context = CmsRenderingContext::new(object!({}), &query_data, &renderer);
  let content = page.content.as_deref().unwrap_or("");
  let resource_descriptor = ResourceDescriptor::Page(page.name.clone());

  check_rendering(
    rendering_context,
    Some(PreloadPartialsStrategy::ByPage(&page)),
    content,
    &query_data,
    resource_descriptor,
  )
  .await
}

async fn render_layout(
  layout: cms_layouts::Model,
  convention: Option<conventions::Model>,
  user: Option<users::Model>,
  root_site: &root_sites::Model,
  connection_wrapper: &ConnectionWrapper,
  schema_data: &SchemaData,
) -> Result<String, LiquidRenderingError> {
  let parent_convention = match layout.parent_type.as_deref() {
    Some("Convention") => convention,
    _ => None,
  };
  let (query_data, renderer) = build_query_data_and_renderer(
    parent_convention,
    root_site,
    user,
    connection_wrapper,
    schema_data,
  )
  .await;
  let rendering_context = CmsRenderingContext::new(
    object!({
      "content_for_head": "",
      "content_for_layout": "",
      "content_for_navbar": ""
    }),
    &query_data,
    &renderer,
  );
  let content = layout.content.as_deref().unwrap_or("");
  let resource_descriptor = ResourceDescriptor::Layout(layout.name.clone());

  check_rendering(
    rendering_context,
    Some(PreloadPartialsStrategy::ByLayout(&layout)),
    content,
    &query_data,
    resource_descriptor,
  )
  .await
}

async fn check_rendering<'a>(
  rendering_context: CmsRenderingContext<'a>,
  preload_partials_strategy: Option<PreloadPartialsStrategy<'a>>,
  content: &str,
  query_data: &'a QueryData,
  resource_descriptor: ResourceDescriptor,
) -> std::result::Result<String, LiquidRenderingError> {
  rendering_context
    .render_liquid(content, preload_partials_strategy)
    .await
    .map_err(|err| LiquidRenderingError {
      convention_name: query_data.convention().and_then(|c| c.name.clone()),
      resource_descriptor,
      user_descriptor: query_data
        .current_user()
        .map(|u| u.name_without_nickname())
        .unwrap_or_else(|| "anonymous user".to_string()),
      error: err,
    })
}

async fn build_query_data_and_renderer(
  parent_convention: Option<conventions::Model>,
  root_site: &root_sites::Model,
  user: Option<users::Model>,
  connection_wrapper: &ConnectionWrapper,
  schema_data: &SchemaData,
) -> (QueryData, IntercodeLiquidRenderer) {
  let cms_parent = match parent_convention.as_ref() {
    Some(convention) => CmsParent::Convention(Box::new(convention.clone())),
    None => CmsParent::RootSite(Box::new(root_site.clone())),
  };
  let current_user = Arc::new(user);
  let user_con_profile = find_or_build_user_con_profile(
    current_user.as_ref().as_ref(),
    parent_convention.as_ref(),
    connection_wrapper,
  )
  .await;
  let query_data = build_query_data(
    cms_parent,
    current_user.as_ref().to_owned(),
    parent_convention,
    connection_wrapper,
    user_con_profile,
  );
  let renderer = IntercodeLiquidRenderer::new(
    &query_data,
    schema_data,
    AuthorizationInfo::new(
      connection_wrapper.clone(),
      current_user.as_ref().clone(),
      None,
      None,
    ),
  );
  (query_data, renderer)
}

pub async fn check_liquid() -> Result<()> {
  setup_tracing(EnvFilter::new("error"));

  let startup_bar = ProgressBar::new_spinner();

  startup_bar.set_message("Connecting to database...");
  let db = Arc::new(connect_database().await?);
  startup_bar.set_message("Loading translations...");
  let schema_data = SchemaData {
    stripe_client: stripe::Client::new("sk_test_XXXXX"),
    language_loader: Arc::new(build_language_loader()?),
  };
  let connection_wrapper = ConnectionWrapper::from(db);
  startup_bar.set_message("Loading root site...");
  let root_site = root_sites::Entity::find()
    .one(&connection_wrapper)
    .await?
    .expect("No root site found in database");

  startup_bar.set_message("Finding admin user...");
  let admin_user = users::Entity::find()
    .filter(users::Column::SiteAdmin.eq(true))
    .one(&connection_wrapper)
    .await?
    .expect("No admin user found in database");

  startup_bar.set_message("Loading CMS pages...");
  let all_pages = pages::Entity::find()
    .join(
      sea_orm::JoinType::LeftJoin,
      pages::Entity::belongs_to(conventions::Entity)
        .from(pages::Column::ParentId)
        .to(conventions::Column::Id)
        .into(),
    )
    .select_also(conventions::Entity)
    .all(connection_wrapper.as_ref())
    .await?;

  startup_bar.set_message("Loading CMS layouts...");
  let all_layouts = cms_layouts::Entity::find()
    .join(
      sea_orm::JoinType::LeftJoin,
      cms_layouts::Entity::belongs_to(conventions::Entity)
        .from(cms_layouts::Column::ParentId)
        .to(conventions::Column::Id)
        .into(),
    )
    .select_also(conventions::Entity)
    .all(connection_wrapper.as_ref())
    .await?;

  startup_bar.finish();

  let mut errors: Vec<LiquidRenderingError> = vec![];

  let mut queue = futures::stream::iter(all_pages.iter())
    .map(|(page, convention)| {
      render_page(
        page.clone(),
        convention.clone(),
        None,
        &root_site,
        &connection_wrapper,
        &schema_data,
      )
      .boxed()
    })
    .chain(
      futures::stream::iter(all_pages.iter()).map(|(page, convention)| {
        render_page(
          page.clone(),
          convention.clone(),
          Some(admin_user.clone()),
          &root_site,
          &connection_wrapper,
          &schema_data,
        )
        .boxed()
      }),
    )
    .chain(
      futures::stream::iter(all_layouts.iter())
        .map(|(layout, convention)| {
          render_layout(
            layout.clone(),
            convention.clone(),
            None,
            &root_site,
            &connection_wrapper,
            &schema_data,
          )
          .boxed()
        })
        .chain(
          futures::stream::iter(all_layouts.iter()).map(|(layout, convention)| {
            render_layout(
              layout.clone(),
              convention.clone(),
              Some(admin_user.clone()),
              &root_site,
              &connection_wrapper,
              &schema_data,
            )
            .boxed()
          }),
        ),
    )
    .buffer_unordered(100);

  let progress_bar = Arc::new(ProgressBar::new(
    ((all_pages.len() + all_layouts.len()) * 2)
      .try_into()
      .unwrap(),
  ));
  progress_bar.set_style(ProgressStyle::with_template(
    "{msg} {wide_bar} {pos}/{len} [{elapsed_precise}]",
  )?);
  progress_bar.set_message("Rendering CMS content...");

  while let Some(result) = queue.next().await {
    progress_bar.inc(1);

    if let Err(err) = result {
      errors.push(err);
    }
  }

  let errors_by_convention_name: HashMap<String, Vec<LiquidRenderingError>> = errors
    .into_iter()
    .map(|err| {
      (
        err
          .convention_name
          .to_owned()
          .unwrap_or_else(|| "root site".to_string()),
        err,
      )
    })
    .into_group_map();

  for (convention_name, errors) in errors_by_convention_name.iter() {
    println!("{}", convention_name);
    println!("{}", "=".repeat(convention_name.len()));
    for error in errors {
      println!(
        "  {} as {}:",
        error.resource_descriptor, error.user_descriptor
      );
      println!("    {}", error.error.message);
    }
  }

  println!(
    "{} total errors across {} conventions",
    errors_by_convention_name
      .values()
      .map(|errors| errors.len())
      .sum::<usize>(),
    errors_by_convention_name.len()
  );

  Ok(())
}
