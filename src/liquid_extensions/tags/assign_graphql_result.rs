use std::{collections::HashMap, io::Write};

use async_graphql::{indexmap::IndexMap, EmptyMutation, EmptySubscription, Variables};
use async_graphql_value::ConstValue;
use liquid::Error;
use liquid_core::error::ResultLiquidExt;
use liquid_core::{
  Expression, Language, ParseTag, Renderable, Result, Runtime, TagReflection, TagTokenIter,
  ValueCow,
};
use sea_orm::{ColumnTrait, QueryFilter};
use tokio::runtime::Handle;

use crate::{
  api, cms_graphql_queries,
  cms_parent::{CmsParent, CmsParentTrait},
  QueryData, SchemaData,
};

fn liquid_value_to_graphql_const(
  value: &liquid_core::Value,
) -> Result<ConstValue, serde_json::Error> {
  serde_json::from_value::<ConstValue>(serde_json::to_value(value)?)
}

fn graphql_const_to_liquid_value(
  value: ConstValue,
) -> Result<liquid_core::Value, serde_json::Error> {
  serde_json::from_value::<liquid_core::Value>(serde_json::to_value(value)?)
}

#[derive(Clone, Debug)]
pub struct AssignGraphQLResultTag {
  cms_parent: Option<CmsParent>,
  query_data: QueryData,
  schema_data: SchemaData,
}

impl AssignGraphQLResultTag {
  pub fn new(
    cms_parent: Option<CmsParent>,
    query_data: &QueryData,
    schema_data: &SchemaData,
  ) -> AssignGraphQLResultTag {
    AssignGraphQLResultTag {
      cms_parent,
      query_data: query_data.to_owned(),
      schema_data: schema_data.to_owned(),
    }
  }
}

impl TagReflection for AssignGraphQLResultTag {
  fn tag(&self) -> &'static str {
    "assign_graphql_result"
  }

  fn description(&self) -> &'static str {
    "Runs a given GraphQL query (defined in the CMS tab \"GraphQL queries\") and assigns the \
    result to a variable."
  }
}

impl ParseTag for AssignGraphQLResultTag {
  fn parse(
    &self,
    mut arguments: TagTokenIter<'_>,
    _options: &Language,
  ) -> Result<Box<dyn Renderable>> {
    let destination = arguments
      .expect_next("Identifier expected.")?
      .expect_identifier()
      .into_result()?
      .to_string();

    arguments
      .expect_next("Assignment operator \"=\" expected.")?
      .expect_str("=")
      .into_result_custom_msg("Assignment operator \"=\" expected.")?;

    let query_name = arguments
      .expect_next("Identifier expected.")?
      .expect_identifier()
      .into_result()?
      .to_string();

    let mut arg_mapping = HashMap::<String, Expression>::new();

    loop {
      let field = arguments.next();

      match field {
        None => break,
        Some(token) => {
          let field = token
            .expect_identifier()
            .into_result_custom_msg("Argument name expected.")?;

          arguments
            .expect_next("Colon \":\" expected.")?
            .expect_str(":")
            .into_result_custom_msg("Colon \":\" expected.")?;

          let value = arguments
            .expect_next("Value expected.")?
            .expect_value()
            .into_result_custom_msg("Value expected.")?;

          arg_mapping.insert(field.to_string(), value);
        }
      }
    }

    arguments.expect_nothing()?;

    let cms_parent = match &self.cms_parent {
      Some(parent) => Ok(parent.to_owned()),
      None => Err(liquid_core::Error::with_msg(
        "assign_graphql_result can only be used inside a CMS parent (root site or convention)",
      )),
    }?;

    Ok(Box::new(AssignGraphQLResult {
      cms_parent,
      schema_data: self.schema_data.clone(),
      query_data: self.query_data.clone(),
      arg_mapping,
      destination,
      query_name,
    }))
  }

  fn reflection(&self) -> &dyn TagReflection {
    self
  }
}

#[derive(Debug)]
struct AssignGraphQLResult {
  cms_parent: CmsParent,
  query_data: QueryData,
  schema_data: SchemaData,
  arg_mapping: HashMap<String, Expression>,
  destination: String,
  query_name: String,
}

impl AssignGraphQLResult {
  fn trace(&self) -> String {
    format!(
      "{{% assign_graphql_result {} = {}({}) %}}",
      self.destination,
      self.query_name,
      self
        .arg_mapping
        .iter()
        .map(|(field, value)| format!("{}: {}", field, value))
        .collect::<Vec<String>>()
        .join(", ")
    )
  }
}

impl Renderable for AssignGraphQLResult {
  fn render_to(&self, _writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
    let db_ref = self.schema_data.db.clone();
    let cms_graphql_queries_select = self
      .cms_parent
      .cms_graphql_queries()
      .filter(cms_graphql_queries::Column::Identifier.eq(self.query_name.clone()));
    let graphql_query = tokio::task::block_in_place(move || {
      Handle::current()
        .block_on(async move { cms_graphql_queries_select.one(db_ref.as_ref()).await })
    })
    .map_err(|err| liquid_core::Error::with_msg(err.to_string()))?
    .ok_or_else(|| {
      liquid_core::Error::with_msg(format!("GraphQL query not found: {}", self.query_name))
    })?;

    let graphql_schema =
      async_graphql::Schema::build(api::QueryRoot, EmptyMutation, EmptySubscription)
        .extension(async_graphql::extensions::Tracing)
        .data(self.schema_data.to_owned())
        .finish();

    let mut variables_map = IndexMap::with_capacity(self.arg_mapping.len());
    let variable_pairs = self
      .arg_mapping
      .iter()
      .map(|(field, value)| Ok((field, value.evaluate(runtime)?)))
      .collect::<Result<Vec<(&String, ValueCow)>, Error>>()?
      .iter()
      .map(|(field, value)| {
        liquid_value_to_graphql_const(&value.as_view().to_value()).map(|value| (async_graphql::Name::new(*field), value))
      })
      .collect::<Result<Vec<(async_graphql::Name, ConstValue)>, serde_json::Error>>()
      .map_err(|e| Error::with_msg(e.to_string()))?;

    for (key, value) in variable_pairs {
      variables_map.insert(key, value);
    }

    let request = async_graphql::Request::new(graphql_query.query.unwrap_or(String::from("")))
      .data(self.query_data.to_owned())
      .variables(Variables::from_value(ConstValue::Object(variables_map)));
    let response = tokio::task::block_in_place(move || {
      Handle::current().block_on(async move { graphql_schema.execute(request).await })
    });

    runtime.set_global(
      liquid_core::model::KString::from_string(self.destination.clone()),
      graphql_const_to_liquid_value(response.data)
        .map_err(|e| Error::with_msg(e.to_string()))
        .trace_with(|| self.trace().into())?,
    );

    if !response.errors.is_empty() {
      let formatted_errors = response
        .errors
        .iter()
        .map(|err| err.to_string())
        .collect::<Vec<String>>()
        .join(", ");
      return Err(Error::with_msg(formatted_errors));
    }

    Ok(())
  }
}
