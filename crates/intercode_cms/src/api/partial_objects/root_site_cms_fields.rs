use std::env;

use async_graphql::*;
use intercode_entities::root_sites;
use intercode_graphql_core::model_backed_type;
use url::Url;

use crate::{
  api::objects::{
    CmsContentType, CmsFileType, CmsGraphqlQueryType, CmsLayoutType, CmsNavigationItemType,
    CmsPartialType, CmsVariableType, LiquidAssignType, PageType,
  },
  cms_parent_implementation::CmsParentImplementation,
};

model_backed_type!(RootSiteCmsFields, root_sites::Model);

impl CmsParentImplementation<root_sites::Model> for RootSiteCmsFields {}

#[Object]
impl RootSiteCmsFields {
  pub async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "site_name")]
  async fn site_name(&self) -> &str {
    self.model.site_name.as_deref().unwrap_or_default()
  }

  async fn cms_files(&self, ctx: &Context<'_>) -> Result<Vec<CmsFileType>, Error> {
    CmsParentImplementation::cms_files(self, ctx).await
  }

  async fn cms_file(&self, ctx: &Context<'_>, id: ID) -> Result<CmsFileType, Error> {
    CmsParentImplementation::cms_file(self, ctx, id).await
  }

  async fn cms_graphql_queries(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsGraphqlQueryType>, Error> {
    CmsParentImplementation::cms_graphql_queries(self, ctx).await
  }

  async fn cms_graphql_query(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<CmsGraphqlQueryType, Error> {
    CmsParentImplementation::cms_graphql_query(self, ctx, id).await
  }

  async fn cms_layouts(&self, ctx: &Context<'_>) -> Result<Vec<CmsLayoutType>, Error> {
    CmsParentImplementation::cms_layouts(self, ctx).await
  }

  async fn cms_layout(&self, ctx: &Context<'_>, id: ID) -> Result<CmsLayoutType, Error> {
    CmsParentImplementation::cms_layout(self, ctx, id).await
  }

  async fn cms_navigation_items(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsNavigationItemType>, Error> {
    CmsParentImplementation::cms_navigation_items(self, ctx).await
  }

  async fn cms_pages(&self, ctx: &Context<'_>) -> Result<Vec<PageType>, Error> {
    CmsParentImplementation::cms_pages(self, ctx).await
  }

  async fn cms_page(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
    slug: Option<String>,
    root_page: Option<bool>,
  ) -> Result<PageType, Error> {
    CmsParentImplementation::cms_page(self, ctx, id, slug, root_page).await
  }

  async fn cms_partials(&self, ctx: &Context<'_>) -> Result<Vec<CmsPartialType>, Error> {
    CmsParentImplementation::cms_partials(self, ctx).await
  }

  async fn cms_variables(&self, ctx: &Context<'_>) -> Result<Vec<CmsVariableType>, Error> {
    CmsParentImplementation::cms_variables(self, ctx).await
  }

  async fn default_layout(&self, ctx: &Context<'_>) -> Result<CmsLayoutType, Error> {
    CmsParentImplementation::default_layout(self, ctx).await
  }

  async fn effective_cms_layout(
    &self,
    ctx: &Context<'_>,
    path: String,
  ) -> Result<CmsLayoutType, Error> {
    CmsParentImplementation::effective_cms_layout(self, ctx, path).await
  }

  async fn host(&self) -> Result<String> {
    Ok(env::var("INTERCODE_HOST")?)
  }

  async fn liquid_assigns(&self, ctx: &Context<'_>) -> Result<Vec<LiquidAssignType>> {
    CmsParentImplementation::liquid_assigns(self, ctx).await
  }

  async fn preview_liquid(&self, ctx: &Context<'_>, content: String) -> Result<String, Error> {
    CmsParentImplementation::preview_liquid(self, ctx, content).await
  }

  async fn preview_markdown(
    &self,
    ctx: &Context<'_>,
    markdown: String,
    event_id: Option<ID>,
    event_proposal_id: Option<ID>,
  ) -> Result<String, Error> {
    CmsParentImplementation::preview_markdown(self, ctx, markdown, event_id, event_proposal_id)
      .await
  }

  async fn root_page(&self, ctx: &Context<'_>) -> Result<PageType, Error> {
    CmsParentImplementation::root_page(self, ctx).await
  }

  async fn typeahead_search_cms_content(
    &self,
    ctx: &Context<'_>,
    name: Option<String>,
  ) -> Result<Vec<CmsContentType>, Error> {
    CmsParentImplementation::typeahead_search_cms_content(self, ctx, name).await
  }

  async fn url(&self) -> Result<String> {
    let host = env::var("INTERCODE_HOST")?;
    Ok(Url::parse(format!("https://{}", host).as_str())?.to_string())
  }
}
