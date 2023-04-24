use std::{collections::HashMap, sync::Arc};

use async_graphql::dataloader::{DataLoader, Loader};
use async_trait::async_trait;
use futures::try_join;
use intercode_entities::{
  cms_content_groups, conventions, event_categories, model_ext::permissions::PermissionedModelRef,
  permissions,
};
use sea_orm::DbErr;
use seawater::{
  loaders::{EntityRelationLoader, ExpectModel},
  ConnectionWrapper,
};
use std::time::Duration;

use super::exclusive_arc_utils::merge_hash_maps;
use crate::exclusive_arc_variant_loader;

pub struct PermissionedModelsLoader {
  permission_cms_content_group_loader:
    DataLoader<EntityRelationLoader<permissions::Entity, cms_content_groups::Entity>>,
  permission_convention_loader:
    DataLoader<EntityRelationLoader<permissions::Entity, conventions::Entity>>,
  permission_event_category_loader:
    DataLoader<EntityRelationLoader<permissions::Entity, event_categories::Entity>>,
}

impl PermissionedModelsLoader {
  pub fn new(db: ConnectionWrapper, delay: Duration) -> Self {
    Self {
      permission_cms_content_group_loader: DataLoader::new(
        EntityRelationLoader::new(db.clone(), permissions::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay),
      permission_convention_loader: DataLoader::new(
        EntityRelationLoader::new(db.clone(), permissions::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay),
      permission_event_category_loader: DataLoader::new(
        EntityRelationLoader::new(db, permissions::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay),
    }
  }
}

#[derive(Clone)]
pub enum PermissionedModel {
  CmsContentGroup(cms_content_groups::Model),
  Convention(conventions::Model),
  EventCategory(event_categories::Model),
}

exclusive_arc_variant_loader!(
  load_cms_content_groups,
  cms_content_groups::Entity,
  PermissionedModelRef,
  PermissionedModelRef::CmsContentGroup,
  PermissionedModel,
  PermissionedModel::CmsContentGroup
);

exclusive_arc_variant_loader!(
  load_conventions,
  conventions::Entity,
  PermissionedModelRef,
  PermissionedModelRef::Convention,
  PermissionedModel,
  PermissionedModel::Convention
);

exclusive_arc_variant_loader!(
  load_event_categories,
  event_categories::Entity,
  PermissionedModelRef,
  PermissionedModelRef::EventCategory,
  PermissionedModel,
  PermissionedModel::EventCategory
);

#[async_trait]
impl Loader<PermissionedModelRef> for PermissionedModelsLoader {
  type Value = PermissionedModel;
  type Error = Arc<DbErr>;

  async fn load(
    &self,
    keys: &[PermissionedModelRef],
  ) -> Result<HashMap<PermissionedModelRef, Self::Value>, Self::Error> {
    let (cms_content_groups, conventions, event_categories) = try_join!(
      load_cms_content_groups(keys, &self.permission_cms_content_group_loader),
      load_conventions(keys, &self.permission_convention_loader),
      load_event_categories(keys, &self.permission_event_category_loader),
    )?;

    Ok(merge_hash_maps(vec![
      cms_content_groups,
      conventions,
      event_categories,
    ]))
  }
}

impl ExpectModel<PermissionedModel> for Option<PermissionedModel> {
  fn expect_model(&self) -> Result<PermissionedModel, async_graphql::Error> {
    if let Some(model) = self {
      Ok(model.to_owned())
    } else {
      Err(async_graphql::Error::new("Permissioned model not found"))
    }
  }
}
