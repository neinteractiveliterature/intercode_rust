use super::{
  CmsContentGroupType, CmsContentType, CmsFileType, CmsGraphqlQueryType, CmsLayoutType,
  CmsNavigationItemType, CmsPartialType, CmsVariableType, PageType,
};
use crate::{api::interfaces::CmsParentImplementation, model_backed_type};
use async_graphql::*;
use intercode_entities::root_sites;

model_backed_type!(RootSiteType, root_sites::Model);

#[Object(name = "RootSite")]
impl RootSiteType {
  pub async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "site_name")]
  async fn site_name(&self) -> Option<&str> {
    self.model.site_name.as_deref()
  }

  async fn cms_content_groups(&self, ctx: &Context<'_>) -> Result<Vec<CmsContentGroupType>, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_content_groups(self, ctx).await
  }

  async fn cms_content_group(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<CmsContentGroupType, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_content_group(self, ctx, id).await
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
}

impl CmsParentImplementation<root_sites::Model> for RootSiteType {}
