use async_graphql::async_trait::async_trait;
use async_graphql::indexmap::IndexMap;
use futures::try_join;
use intercode_entities::{conventions, events};
use intercode_graphql::{
  build_partial_compiler, EmbeddedGraphQLExecutorBuilder, LiquidRenderer, QueryData, SchemaData,
};
use intercode_liquid::{build_liquid_parser, cms_parent_partial_source::PreloadPartialsStrategy};
use intercode_policies::AuthorizationInfo;
use liquid::ValueView;
use once_cell::race::OnceBox;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use seawater::{liquid_drop_impl, DropResult, ExtendedDropResult};
use seawater::{Context, DropError, DropStore, ModelBackedDrop};
use std::{
  fmt::Debug,
  sync::{Arc, Weak},
};

use crate::drops::{ConventionDrop, DropContext, EventDrop, UserConProfileDrop};

#[derive(Debug)]
struct IntercodeGlobals {
  query_data: QueryData,
  context: DropContext,
  _liquid_object_view_pairs: OnceBox<IndexMap<String, Box<dyn ValueView + Send + Sync>>>,
}

impl Clone for IntercodeGlobals {
  fn clone(&self) -> Self {
    Self {
      query_data: self.query_data.clone(),
      context: self.context.clone(),
      _liquid_object_view_pairs: OnceBox::new(),
    }
  }
}

#[liquid_drop_impl(i64, DropContext)]
impl IntercodeGlobals {
  pub fn new(query_data: QueryData, schema_data: SchemaData, store: Weak<DropStore<i64>>) -> Self {
    IntercodeGlobals {
      query_data: query_data.clone(),
      context: DropContext::new(schema_data, query_data, store),
      _liquid_object_view_pairs: OnceBox::new(),
    }
  }

  fn id(&self) -> i64 {
    0
  }

  fn convention(&self) -> Option<ConventionDrop> {
    self
      .query_data
      .convention()
      .map(|convention| ConventionDrop::new(convention.clone(), self.context.clone()))
  }

  async fn conventions(&self) -> Result<Vec<ConventionDrop>, DropError> {
    Ok(
      conventions::Entity::find()
        .filter(conventions::Column::Hidden.eq(false))
        .all(self.context.db())
        .await?
        .iter()
        .map(|convention| ConventionDrop::new(convention.clone(), self.context.clone()))
        .collect(),
    )
  }

  async fn event(&self) -> Option<EventDrop> {
    if let Some(convention) = self.query_data.convention() {
      if convention.site_mode == "single_event" {
        return events::Entity::find()
          .filter(events::Column::ConventionId.eq(convention.id))
          .one(self.context.db())
          .await
          .ok()
          .flatten()
          .map(|event| EventDrop::new(event, self.context.clone()));
      }
    }

    None
  }

  async fn user_con_profile(&self) -> Option<UserConProfileDrop> {
    let ucp = self.query_data.user_con_profile().map(|user_con_profile| {
      UserConProfileDrop::new(user_con_profile.clone(), self.context.clone())
    });

    if let Some(ucp) = ucp {
      let ucp_ref = self
        .context
        .with_drop_store(|store| store.store(ucp.clone()));
      let drops = vec![ucp_ref];
      try_join!(
        UserConProfileDrop::preload_signups(self.context.clone(), &drops),
        UserConProfileDrop::preload_staff_positions(self.context.clone(), &drops),
        UserConProfileDrop::preload_ticket(self.context.clone(), &drops),
        UserConProfileDrop::preload_user(self.context.clone(), &drops),
      );
      Some(ucp)
    } else {
      None
    }
  }
}

#[derive(Debug, Clone)]
pub struct IntercodeLiquidRenderer {
  query_data: QueryData,
  schema_data: SchemaData,
  authorization_info: AuthorizationInfo,
  store: Arc<DropStore<i64>>,
}

impl IntercodeLiquidRenderer {
  pub fn new(
    query_data: &QueryData,
    schema_data: &SchemaData,
    authorization_info: AuthorizationInfo,
  ) -> Self {
    IntercodeLiquidRenderer {
      query_data: query_data.clone(),
      schema_data: schema_data.clone(),
      authorization_info,
      store: Default::default(),
    }
  }
}

#[async_trait]
impl LiquidRenderer for IntercodeLiquidRenderer {
  async fn builtin_globals(
    &self,
  ) -> Result<Box<dyn liquid::ObjectView + Send>, async_graphql::Error> {
    let schema_data: SchemaData = self.schema_data.clone();
    let query_data: QueryData = self.query_data.clone();
    Ok(Box::new(IntercodeGlobals::new(
      query_data,
      schema_data,
      Arc::downgrade(&self.store),
    )))
  }

  async fn render_liquid(
    &self,
    content: &str,
    globals: liquid::Object,
    preload_partials_strategy: Option<PreloadPartialsStrategy<'_>>,
  ) -> Result<String, async_graphql::Error> {
    let schema_data: SchemaData = self.schema_data.clone();
    let query_data: QueryData = self.query_data.clone();
    let cms_parent = query_data.cms_parent().clone();

    let partial_compiler = build_partial_compiler(
      cms_parent,
      query_data.db().clone(),
      preload_partials_strategy,
    )
    .await?;
    let user_signed_in = query_data.current_user().is_some();
    let executor_builder = EmbeddedGraphQLExecutorBuilder::new(
      query_data.clone(),
      schema_data.clone(),
      self.authorization_info.clone(),
    );

    let parser = build_liquid_parser(
      query_data.convention(),
      Arc::downgrade(&schema_data.language_loader),
      query_data.cms_parent(),
      query_data.db().clone(),
      user_signed_in,
      Box::new(executor_builder),
      partial_compiler,
    )?;

    let builtins = DropResult::new(IntercodeGlobals::new(
      query_data,
      schema_data,
      Arc::downgrade(&self.store),
    ));
    let globals_with_builtins = builtins.extend(globals);

    let template = parser.parse(content)?;
    let result =
      tokio::task::spawn_blocking(move || template.render(&globals_with_builtins)).await?;

    match result {
      Ok(content) => Ok(content),
      Err(error) => Err(async_graphql::Error::new(error.to_string())),
    }
  }
}
