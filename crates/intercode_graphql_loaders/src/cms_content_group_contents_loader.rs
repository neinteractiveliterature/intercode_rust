use std::{collections::HashMap, sync::Arc, time::Duration};

use async_graphql::dataloader::{DataLoader, Loader};
use async_trait::async_trait;
use futures::try_join;
use intercode_entities::{cms_content_group_associations, cms_layouts, cms_partials, pages};
use itertools::Itertools;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use seawater::{loaders::EntityRelationLoader, ConnectionWrapper};

use crate::exclusive_arc_utils::loader_result_hashmap_to_model_hashmap;

use super::exclusive_arc_utils::merge_hash_maps;

pub struct CmsContentGroupContentsLoader {
  db: ConnectionWrapper,
  cms_content_group_association_cms_layout_loader:
    DataLoader<EntityRelationLoader<cms_content_group_associations::Entity, cms_layouts::Entity>>,
  cms_content_group_association_cms_partial_loader:
    DataLoader<EntityRelationLoader<cms_content_group_associations::Entity, cms_partials::Entity>>,
  cms_content_group_association_page_loader:
    DataLoader<EntityRelationLoader<cms_content_group_associations::Entity, pages::Entity>>,
}

#[derive(Clone)]
pub enum CmsContentGroupItem {
  Page(pages::Model),
  CmsLayout(cms_layouts::Model),
  CmsPartial(cms_partials::Model),
}

impl From<pages::Model> for CmsContentGroupItem {
  fn from(value: pages::Model) -> Self {
    CmsContentGroupItem::Page(value)
  }
}

impl From<cms_layouts::Model> for CmsContentGroupItem {
  fn from(value: cms_layouts::Model) -> Self {
    CmsContentGroupItem::CmsLayout(value)
  }
}

impl From<cms_partials::Model> for CmsContentGroupItem {
  fn from(value: cms_partials::Model) -> Self {
    CmsContentGroupItem::CmsPartial(value)
  }
}

impl CmsContentGroupContentsLoader {
  pub fn new(db: ConnectionWrapper, delay: Duration) -> Self {
    CmsContentGroupContentsLoader {
      db: db.clone(),
      cms_content_group_association_cms_layout_loader: DataLoader::new(
        EntityRelationLoader::new(db.clone(), cms_content_group_associations::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay),
      cms_content_group_association_cms_partial_loader: DataLoader::new(
        EntityRelationLoader::new(db.clone(), cms_content_group_associations::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay),
      cms_content_group_association_page_loader: DataLoader::new(
        EntityRelationLoader::new(db, cms_content_group_associations::PrimaryKey::Id),
        tokio::spawn,
      )
      .delay(delay),
    }
  }
}

#[async_trait]
impl Loader<i64> for CmsContentGroupContentsLoader {
  type Value = Vec<CmsContentGroupItem>;
  type Error = Arc<DbErr>;

  async fn load(
    &self,
    keys: &[i64],
  ) -> Result<std::collections::HashMap<i64, Self::Value>, Self::Error> {
    let associations = cms_content_group_associations::Entity::find()
      .filter(cms_content_group_associations::Column::CmsContentGroupId.is_in(keys.iter().copied()))
      .all(&self.db)
      .await?;
    let association_ids = associations
      .iter()
      .map(|association| association.id)
      .collect::<Vec<_>>();

    let (cms_layouts, cms_partials, pages) = try_join!(
      async {
        Ok::<_, Arc<DbErr>>(
          loader_result_hashmap_to_model_hashmap(
            self
              .cms_content_group_association_cms_layout_loader
              .load_many(association_ids.clone())
              .await?,
          )
          .into_iter()
          .map(|(key, model)| (key, CmsContentGroupItem::from(model)))
          .collect::<HashMap<_, _>>(),
        )
      },
      async {
        Ok::<_, Arc<DbErr>>(
          loader_result_hashmap_to_model_hashmap(
            self
              .cms_content_group_association_cms_partial_loader
              .load_many(association_ids.clone())
              .await?,
          )
          .into_iter()
          .map(|(key, model)| (key, CmsContentGroupItem::from(model)))
          .collect::<HashMap<_, _>>(),
        )
      },
      async {
        Ok::<_, Arc<DbErr>>(
          loader_result_hashmap_to_model_hashmap(
            self
              .cms_content_group_association_page_loader
              .load_many(association_ids.clone())
              .await?,
          )
          .into_iter()
          .map(|(key, model)| (key, CmsContentGroupItem::from(model)))
          .collect::<HashMap<_, _>>(),
        )
      },
    )?;

    let content_by_association_id = merge_hash_maps(vec![cms_layouts, cms_partials, pages]);

    Ok(
      associations
        .iter()
        .filter_map(|association| {
          content_by_association_id
            .get(&association.id)
            .map(|content| (association.cms_content_group_id, content.clone()))
        })
        .into_group_map(),
    )
  }
}
