use async_graphql::async_trait::async_trait;
use intercode_entities::conventions;
use intercode_graphql::{LiquidRenderer, QueryData, SchemaData};
use intercode_liquid::{build_liquid_parser, cms_parent_partial_source::PreloadPartialsStrategy};
use lazy_liquid_value_view::{liquid_drop_impl, liquid_drop_struct};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use seawater::{Context, DropError, ModelBackedDrop};
use std::fmt::Debug;

use crate::drops::{ConventionDrop, DropContext, UserConProfileDrop};

#[liquid_drop_struct]
struct IntercodeGlobals {
  query_data: QueryData,
  drop_context: DropContext,
}

#[liquid_drop_impl]
impl IntercodeGlobals {
  pub fn new(query_data: QueryData, schema_data: SchemaData) -> Self {
    IntercodeGlobals {
      query_data,
      drop_context: DropContext::new(schema_data),
    }
  }

  fn convention(&self) -> Option<ConventionDrop> {
    self
      .query_data
      .convention
      .as_ref()
      .as_ref()
      .map(|convention| ConventionDrop::new(convention.clone(), self.drop_context.clone()))
  }

  async fn conventions(&self) -> Result<Vec<ConventionDrop>, DropError> {
    Ok(
      conventions::Entity::find()
        .filter(conventions::Column::Hidden.eq(false))
        .all(self.drop_context.db())
        .await?
        .iter()
        .map(|convention| ConventionDrop::new(convention.clone(), self.drop_context.clone()))
        .collect(),
    )
  }

  fn user_con_profile(&self) -> Option<UserConProfileDrop> {
    self
      .query_data
      .user_con_profile
      .as_ref()
      .as_ref()
      .map(|user_con_profile| {
        UserConProfileDrop::new(user_con_profile.clone(), self.drop_context.clone())
      })
  }
}

#[derive(Debug, Clone)]
pub struct IntercodeLiquidRenderer {
  query_data: QueryData,
  schema_data: SchemaData,
}

impl IntercodeLiquidRenderer {
  pub fn new(query_data: &QueryData, schema_data: &SchemaData) -> Self {
    IntercodeLiquidRenderer {
      query_data: query_data.clone(),
      schema_data: schema_data.clone(),
    }
  }
}

#[async_trait]
impl LiquidRenderer for IntercodeLiquidRenderer {
  async fn render_liquid(
    &self,
    content: &str,
    globals: liquid::Object,
    preload_partials_strategy: Option<PreloadPartialsStrategy<'_>>,
  ) -> Result<String, async_graphql::Error> {
    let schema_data: SchemaData = self.schema_data.clone();
    let query_data: QueryData = self.query_data.clone();

    let partial_compiler = query_data
      .build_partial_compiler(&schema_data, preload_partials_strategy)
      .await?;
    let convention = query_data.convention.clone();
    let language_loader = schema_data.language_loader.clone();
    let cms_parent = query_data.cms_parent.clone();
    let db = schema_data.db.clone();
    let user_signed_in = query_data.current_user.is_some();
    let executor = query_data.build_embedded_graphql_executor(&schema_data);

    let parser = build_liquid_parser(
      &convention,
      &language_loader,
      &cms_parent,
      &db,
      user_signed_in,
      executor,
      partial_compiler,
    )?;

    let builtins = IntercodeGlobals::new(query_data, schema_data);
    let globals_with_builtins = builtins.extend(globals);

    let template = parser.parse(content)?;
    let result = template.render(&globals_with_builtins);

    match result {
      Ok(content) => Ok(content),
      Err(error) => Err(async_graphql::Error::new(error.to_string())),
    }
  }
}
