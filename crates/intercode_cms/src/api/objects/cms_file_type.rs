use std::sync::Arc;

use async_graphql::*;
use intercode_entities::cms_files;
use intercode_graphql_core::{
  model_backed_type, objects::ActiveStorageAttachmentType, ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{AuthorizationInfo, Policy, ReadManageAction};

use crate::api::policies::CmsContentPolicy;

model_backed_type!(CmsFileType, cms_files::Model);

#[Object(name = "CmsFile")]
impl CmsFileType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "current_ability_can_delete")]
  async fn current_ability_can_delete(&self, ctx: &Context<'_>) -> Result<bool> {
    Ok(
      CmsContentPolicy::action_permitted(
        ctx.data::<AuthorizationInfo>()?,
        &ReadManageAction::Manage,
        &self.model,
      )
      .await?,
    )
  }

  async fn file(&self, ctx: &Context<'_>) -> Result<Option<ActiveStorageAttachmentType>> {
    Ok(
      ctx
        .data::<Arc<LoaderManager>>()?
        .cms_file_file
        .load_one(self.model.id)
        .await?
        .and_then(|files| files.get(0).cloned())
        .map(ActiveStorageAttachmentType::new),
    )
  }
}
