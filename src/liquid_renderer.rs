use async_graphql::async_trait::async_trait;
use intercode_graphql::{loaders::expect::ExpectModels, LiquidRenderer, QueryData, SchemaData};
use intercode_liquid::{
  build_liquid_parser,
  cms_parent_partial_source::PreloadPartialsStrategy,
  drops::{ConventionDrop, UserConProfileDrop},
};
use liquid::object;
use std::fmt::Debug;

#[derive(Debug)]
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
    let signups = if let Some(user_con_profile) = query_data.user_con_profile.as_ref().as_ref() {
      schema_data
        .loaders
        .user_con_profile_signups
        .load_one(user_con_profile.id)
        .await?
        .expect_models()?
        .to_owned()
    } else {
      vec![]
    };

    let parser = build_liquid_parser(
      &convention,
      &language_loader,
      &cms_parent,
      &db,
      user_signed_in,
      executor,
      partial_compiler,
    )?;

    let mut all_globals = object!({
      "convention": query_data.convention.as_ref().as_ref().map(|convention| ConventionDrop::new(convention, language_loader.as_ref())),
      "user_con_profile": query_data.user_con_profile.as_ref().as_ref().map(|ucp| {
        UserConProfileDrop::new(ucp, query_data.current_user.as_ref().as_ref().unwrap(), Box::new(signups.into_iter()))
      })
    });
    all_globals.extend(globals);

    let template = parser.parse(content)?;
    let result = template.render(&all_globals);

    match result {
      Ok(content) => Ok(content),
      Err(error) => Err(async_graphql::Error::new(error.to_string())),
    }
  }
}
