use async_graphql::{Context, Error, Object, ID};
use intercode_cms::{api::partial_objects::RootSiteCmsFields, CmsParentImplementation};
use intercode_entities::root_sites;
use intercode_graphql_core::model_backed_type;

use crate::merged_model_backed_type;

use super::CmsContentGroupType;

model_backed_type!(RootSiteGlueFields, root_sites::Model);

impl CmsParentImplementation<root_sites::Model> for RootSiteGlueFields {}

#[Object]
impl RootSiteGlueFields {
  async fn cms_content_groups(&self, ctx: &Context<'_>) -> Result<Vec<CmsContentGroupType>, Error> {
    CmsParentImplementation::cms_content_groups(self, ctx).await
  }

  pub async fn cms_content_group(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<CmsContentGroupType, Error> {
    CmsParentImplementation::cms_content_group(self, ctx, id).await
  }
}

merged_model_backed_type!(
  RootSiteType,
  root_sites::Model,
  "RootSite",
  RootSiteCmsFields,
  RootSiteGlueFields
);
