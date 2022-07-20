use async_graphql::async_trait::async_trait;
use intercode_graphql::{LiquidRenderer, QueryData, SchemaData};
use intercode_liquid::{build_liquid_parser, cms_parent_partial_source::PreloadPartialsStrategy};
use liquid::object;
use std::fmt::Debug;

use crate::drops::{ConventionDrop, UserConProfileDrop};

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

    let convention_drop = convention.as_ref().as_ref().map(|convention| {
      ConventionDrop::new(
        schema_data.clone(),
        convention.clone(),
        language_loader.clone(),
      )
    });
    let user_con_profile_drop =
      query_data
        .user_con_profile
        .as_ref()
        .as_ref()
        .map(|user_con_profile| {
          UserConProfileDrop::new(user_con_profile.clone(), schema_data.clone())
        });

    let mut globals = globals.clone();
    globals.extend(object!({
      "convention": convention_drop,
      "user_con_profile": user_con_profile_drop
    }));

    let template = parser.parse(content)?;
    let result = template.render(&globals);

    match result {
      Ok(content) => Ok(content),
      Err(error) => Err(async_graphql::Error::new(error.to_string())),
    }
  }
}
