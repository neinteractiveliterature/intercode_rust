use std::{collections::HashMap, sync::Arc};

use async_graphql::Request;
use async_graphql_value::Variables;
use intercode_cms::api::policies::PagePolicy;
use intercode_entities::{
  cms_parent::{CmsParent, CmsParentTrait},
  pages,
};
use intercode_graphql::actions::IntercodeSchema;
use intercode_graphql_core::{
  liquid_renderer::LiquidRenderer,
  query_data::{QueryData, QueryDataContainer},
};
use intercode_graphql_loaders::LoaderManager;
use intercode_graphql_presend_macros::load_operations;
use intercode_policies::{AuthorizationInfo, EntityPolicy, ReadManageAction};
use once_cell::sync::Lazy;
use regex::Regex;
use sea_orm::{ColumnTrait, PaginatorTrait, QueryFilter};
use serde_json::json;
use tracing::error;

#[derive(Clone)]
pub struct GraphQLOperation {
  pub document: String,
  pub ast: serde_json::Value,
}

static PRESENDABLE_OPERATIONS: Lazy<HashMap<String, GraphQLOperation>> = Lazy::new(|| {
  load_operations!(
    "AppRootQuery",
    "CmsPageQuery",
    "CommonConventionDataQuery",
    "PageAdminDropdownQuery",
  )
});

static APP_ROOT_QUERY: Lazy<&GraphQLOperation> =
  Lazy::new(|| PRESENDABLE_OPERATIONS.get("AppRootQuery").unwrap());
static CMS_PAGE_QUERY: Lazy<&GraphQLOperation> =
  Lazy::new(|| PRESENDABLE_OPERATIONS.get("CmsPageQuery").unwrap());
static COMMON_CONVENTION_DATA_QUERY: Lazy<&GraphQLOperation> = Lazy::new(|| {
  PRESENDABLE_OPERATIONS
    .get("CommonConventionDataQuery")
    .unwrap()
});
static PAGE_ADMIN_DROPDOWN_QUERY: Lazy<&GraphQLOperation> = Lazy::new(|| {
  PRESENDABLE_OPERATIONS
    .get("PageAdminDropdownQuery")
    .unwrap()
});

static PAGE_PATH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("^/pages/(.+)$").unwrap());

struct PresendableOperation {
  operation: &'static GraphQLOperation,
  variables: Variables,
}

impl PresendableOperation {
  pub fn new(operation: &'static GraphQLOperation, variables: serde_json::Value) -> Self {
    PresendableOperation {
      operation,
      variables: Variables::from_json(variables),
    }
  }

  pub fn to_request(&self) -> Request {
    Request::new(self.operation.document.clone()).variables(self.variables.clone())
  }
}

pub async fn get_presend_data(
  schema: &IntercodeSchema,
  query_data: QueryData,
  liquid_renderer: Arc<dyn LiquidRenderer>,
  authorization_info: AuthorizationInfo,
  cms_parent: &CmsParent,
  path: &str,
) -> Result<serde_json::Value, async_graphql::Error> {
  let operations = if path == "/" {
    cms_page_queries(path, &authorization_info, cms_parent, &query_data, None).await?
  } else if let Some(captures) = PAGE_PATH_REGEX.captures(path) {
    let slug = captures.get(1).map_or("", |m| m.as_str());
    cms_page_queries(
      path,
      &authorization_info,
      cms_parent,
      &query_data,
      Some(slug),
    )
    .await?
  } else if path.starts_with("/events") {
    events_app_queries()
  } else {
    non_cms_app_root_queries()
  };

  let requests = operations
    .iter()
    .map(PresendableOperation::to_request)
    .collect::<Vec<_>>();

  let loader_manager = Arc::new(LoaderManager::new(query_data.db().clone()));
  let batch_request = async_graphql::BatchRequest::Batch(requests)
    .data(query_data)
    .data(loader_manager)
    .data::<Arc<dyn LiquidRenderer>>(liquid_renderer)
    .data(authorization_info);

  let batch_response = schema.execute_batch(batch_request).await;

  let responses = match batch_response {
    async_graphql::BatchResponse::Single(response) => vec![response],
    async_graphql::BatchResponse::Batch(responses) => responses,
  };

  let query_data = responses
    .into_iter()
    .zip(operations)
    .filter_map(|(response, operation)| {
      if response.is_ok() {
        Some(json!({
          "query": operation.operation.ast,
          "variables": operation.variables,
          "data": response.data
        }))
      } else {
        error!(
          "Errors while pre-sending GraphQL query:\n{:?}",
          response.errors
        );
        None
      }
    })
    .collect::<Vec<_>>();

  Ok(query_data.into())
}

fn non_cms_app_root_queries() -> Vec<PresendableOperation> {
  vec![PresendableOperation::new(
    &APP_ROOT_QUERY,
    json!({ "path": "/non_cms_path" }),
  )]
}

async fn cms_page_queries(
  path: &str,
  authorization_info: &AuthorizationInfo,
  cms_parent: &CmsParent,
  query_data: &Box<dyn QueryDataContainer>,
  slug: Option<&str>,
) -> Result<Vec<PresendableOperation>, async_graphql::Error> {
  let cms_admin = PagePolicy::accessible_to(authorization_info, &ReadManageAction::Manage)
    .count(query_data.db())
    .await?
    > 0;

  let mut operations: Vec<PresendableOperation> = Vec::with_capacity(3);

  operations.push(PresendableOperation::new(
    &APP_ROOT_QUERY,
    json!({ "path": path }),
  ));

  if let Some(slug) = slug {
    operations.push(PresendableOperation::new(
      &CMS_PAGE_QUERY,
      json!({ "slug": slug }),
    ));
  } else {
    operations.push(PresendableOperation::new(
      &CMS_PAGE_QUERY,
      json!({ "rootPage": true }),
    ));
  }

  if cms_admin {
    let page = if let Some(slug) = slug {
      cms_parent
        .pages()
        .filter(pages::Column::Slug.eq(slug))
        .one(query_data.db())
        .await?
    } else {
      cms_parent.root_page().one(query_data.db()).await?
    };

    if let Some(page) = page {
      operations.push(PresendableOperation::new(
        &PAGE_ADMIN_DROPDOWN_QUERY,
        json!({ "id": page.id.to_string() }),
      ))
    }
  }

  Ok(operations)
}

fn events_app_queries() -> Vec<PresendableOperation> {
  vec![
    PresendableOperation::new(&APP_ROOT_QUERY, json!({ "path": "/non_cms_path" })),
    PresendableOperation::new(&COMMON_CONVENTION_DATA_QUERY, json!({})),
  ]
}
