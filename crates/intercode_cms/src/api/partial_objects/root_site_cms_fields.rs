use std::env;

use async_graphql::*;
use intercode_entities::root_sites;
use intercode_graphql_core::model_backed_type;
use url::Url;

use crate::{
  api::objects::{
    CmsContentType, CmsFileType, CmsGraphqlQueryType, CmsLayoutType, CmsNavigationItemType,
    CmsPartialType, CmsVariableType, PageType,
  },
  cms_parent_implementation::CmsParentImplementation,
};

use super::CmsContentGroupCmsFields;

model_backed_type!(RootSiteCmsFields, root_sites::Model);

impl RootSiteCmsFields {
  pub async fn cms_content_groups(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsContentGroupCmsFields>, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_content_groups(self, ctx).await
  }

  pub async fn cms_content_group(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<CmsContentGroupCmsFields, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_content_group(self, ctx, id).await
  }
}

#[Object]
impl RootSiteCmsFields {
  pub async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "site_name")]
  async fn site_name(&self) -> Option<&str> {
    self.model.site_name.as_deref()
  }

  async fn cms_files(&self, ctx: &Context<'_>) -> Result<Vec<CmsFileType>, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_files(self, ctx).await
  }

  async fn cms_file(&self, ctx: &Context<'_>, id: ID) -> Result<CmsFileType, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_file(self, ctx, id).await
  }

  async fn cms_graphql_queries(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsGraphqlQueryType>, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_graphql_queries(self, ctx).await
  }

  async fn cms_graphql_query(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<CmsGraphqlQueryType, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_graphql_query(self, ctx, id).await
  }

  async fn cms_layouts(&self, ctx: &Context<'_>) -> Result<Vec<CmsLayoutType>, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_layouts(self, ctx).await
  }

  async fn cms_layout(&self, ctx: &Context<'_>, id: ID) -> Result<CmsLayoutType, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_layout(self, ctx, id).await
  }

  async fn cms_navigation_items(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsNavigationItemType>, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_navigation_items(self, ctx).await
  }

  async fn cms_pages(&self, ctx: &Context<'_>) -> Result<Vec<PageType>, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_pages(self, ctx).await
  }

  async fn cms_page(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
    slug: Option<String>,
    root_page: Option<bool>,
  ) -> Result<PageType, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_page(self, ctx, id, slug, root_page)
      .await
  }

  async fn cms_partials(&self, ctx: &Context<'_>) -> Result<Vec<CmsPartialType>, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_partials(self, ctx).await
  }

  async fn cms_variables(&self, ctx: &Context<'_>) -> Result<Vec<CmsVariableType>, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_variables(self, ctx).await
  }

  async fn default_layout(&self, ctx: &Context<'_>) -> Result<CmsLayoutType, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::default_layout(self, ctx).await
  }

  async fn effective_cms_layout(
    &self,
    ctx: &Context<'_>,
    path: String,
  ) -> Result<CmsLayoutType, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::effective_cms_layout(self, ctx, path)
      .await
  }

  async fn root_page(&self, ctx: &Context<'_>) -> Result<PageType, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::root_page(self, ctx).await
  }

  async fn typeahead_search_cms_content(
    &self,
    ctx: &Context<'_>,
    name: Option<String>,
  ) -> Result<Vec<CmsContentType>, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::typeahead_search_cms_content(
      self, ctx, name,
    )
    .await
  }

  async fn url(&self) -> Result<String> {
    let host = env::var("INTERCODE_HOST")?;
    Ok(Url::parse(format!("https://{}", host).as_str())?.to_string())
  }
}

impl CmsParentImplementation<root_sites::Model> for RootSiteCmsFields {}
