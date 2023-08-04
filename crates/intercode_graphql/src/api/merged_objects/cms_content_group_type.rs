use async_graphql::*;
use intercode_cms::api::partial_objects::CmsContentGroupCmsFields;
use intercode_entities::cms_content_groups;
use intercode_graphql_core::{model_backed_type, ModelBackedType};

use crate::{api::merged_objects::PermissionType, merged_model_backed_type};

model_backed_type!(CmsContentGroupGlueFields, cms_content_groups::Model);

#[Object]
impl CmsContentGroupGlueFields {
  async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<PermissionType>> {
    CmsContentGroupCmsFields::from_type(self.clone())
      .permissions(ctx)
      .await
      .map(|res| res.into_iter().map(PermissionType::new).collect())
  }
}

merged_model_backed_type!(
  CmsContentGroupType,
  cms_content_groups::Model,
  "CmsContentGroup",
  CmsContentGroupCmsFields,
  CmsContentGroupGlueFields
);
