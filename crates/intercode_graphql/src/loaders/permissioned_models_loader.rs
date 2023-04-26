use std::{collections::HashMap, sync::Arc};

use async_graphql::dataloader::{DataLoader, Loader};
use async_trait::async_trait;
use futures::try_join;
use intercode_entities::{
  cms_content_groups, conventions, event_categories, model_ext::permissions::PermissionedModelRef,
};
use sea_orm::DbErr;
use seawater::{
  loaders::{EntityIdLoader, ExpectModel},
  ConnectionWrapper,
};
use std::time::Duration;

use super::exclusive_arc_utils::merge_hash_maps;
use crate::exclusive_arc_variant_loader;

pub struct PermissionedModelsLoader {
  cms_content_group_loader: DataLoader<EntityIdLoader<cms_content_groups::Entity>>,
  convention_loader: DataLoader<EntityIdLoader<conventions::Entity>>,
  event_category_loader: DataLoader<EntityIdLoader<event_categories::Entity>>,
}

impl PermissionedModelsLoader {
  pub fn new(db: ConnectionWrapper, delay: Duration) -> Self {
    Self {
      cms_content_group_loader: DataLoader::new(
        EntityIdLoader::new(db.clone(), cms_content_groups::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay),
      convention_loader: DataLoader::new(
        EntityIdLoader::new(db.clone(), conventions::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay),
      event_category_loader: DataLoader::new(
        EntityIdLoader::new(db, event_categories::PrimaryKey::Id),
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
      load_cms_content_groups(keys, &self.cms_content_group_loader),
      load_conventions(keys, &self.convention_loader),
      load_event_categories(keys, &self.event_category_loader),
    )?;

    Ok(merge_hash_maps(vec![
      cms_content_groups,
      conventions,
      event_categories,
    ]))
  }
}

impl ExpectModel<PermissionedModel> for Option<PermissionedModel> {
  fn try_one(&self) -> Option<&PermissionedModel> {
    self.as_ref()
  }

  fn expect_one(&self) -> Result<&PermissionedModel, async_graphql::Error> {
    if let Some(model) = self {
      Ok(model)
    } else {
      Err(async_graphql::Error::new("Permissioned model not found"))
    }
  }
}
