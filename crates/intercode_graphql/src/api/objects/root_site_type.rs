use super::{CmsLayoutType, CmsNavigationItemType, PageType};
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

  async fn cms_navigation_items(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsNavigationItemType>, Error> {
    <Self as CmsParentImplementation<root_sites::Model>>::cms_navigation_items(self, ctx).await
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
}

impl CmsParentImplementation<root_sites::Model> for RootSiteType {}
