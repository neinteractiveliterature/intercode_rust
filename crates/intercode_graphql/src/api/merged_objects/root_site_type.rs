use async_graphql::{Context, Error, Object, ID};
use intercode_cms::api::partial_objects::RootSiteCmsFields;
use intercode_entities::root_sites;
use intercode_graphql_core::{model_backed_type, ModelBackedType};

use crate::merged_model_backed_type;

use super::CmsContentGroupType;

model_backed_type!(RootSiteGlueFields, root_sites::Model);

#[Object]
impl RootSiteGlueFields {
  async fn cms_content_groups(&self, ctx: &Context<'_>) -> Result<Vec<CmsContentGroupType>, Error> {
    RootSiteCmsFields::from_type(self.clone())
      .cms_content_groups(ctx)
      .await
      .map(|partials| {
        partials
          .into_iter()
          .map(CmsContentGroupType::from_type)
          .collect()
      })
  }

  pub async fn cms_content_group(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<CmsContentGroupType, Error> {
    RootSiteCmsFields::from_type(self.clone())
      .cms_content_group(ctx, id)
      .await
      .map(CmsContentGroupType::from_type)
  }
}

merged_model_backed_type!(
  RootSiteType,
  root_sites::Model,
  "RootSite",
  RootSiteCmsFields,
  RootSiteGlueFields
);
