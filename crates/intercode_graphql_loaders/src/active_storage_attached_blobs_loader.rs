use async_graphql::{async_trait, dataloader::Loader};
use intercode_entities::{active_storage_attachments, active_storage_blobs};
use sea_orm::{ColumnTrait, DbErr, QueryFilter, Select};
use seawater::ConnectionWrapper;
use std::{collections::HashMap, sync::Arc};

pub struct ActiveStorageAttachedBlobsLoader {
  db: ConnectionWrapper,
  scope: Select<active_storage_attachments::Entity>,
}

impl ActiveStorageAttachedBlobsLoader {
  pub fn new(db: ConnectionWrapper, scope: Select<active_storage_attachments::Entity>) -> Self {
    ActiveStorageAttachedBlobsLoader { db, scope }
  }
}

#[async_trait::async_trait]
impl Loader<i64> for ActiveStorageAttachedBlobsLoader {
  type Value = Vec<active_storage_blobs::Model>;
  type Error = Arc<DbErr>;

  async fn load(
    &self,
    keys: &[i64],
  ) -> Result<std::collections::HashMap<i64, Self::Value>, Self::Error> {
    Ok(
      self
        .scope
        .clone()
        .find_also_related(active_storage_blobs::Entity)
        .filter(active_storage_attachments::Column::RecordId.is_in(keys.iter().copied()))
        .all(&self.db)
        .await?
        .into_iter()
        .fold(
          HashMap::<i64, Self::Value>::with_capacity(keys.len()),
          |mut acc, (attachment, blob)| {
            let attachments = acc.entry(attachment.record_id).or_default();
            if let Some(blob) = blob {
              attachments.push(blob);
            }
            acc
          },
        ),
    )
  }
}
