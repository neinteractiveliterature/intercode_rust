use async_graphql::{async_trait, dataloader::Loader};
use intercode_entities::{
  active_storage_attachments, active_storage_blobs, model_ext::FormResponse,
};
use sea_orm::{ColumnTrait, DbErr, QueryFilter, Select};
use seawater::ConnectionWrapper;
use std::{collections::HashMap, marker::PhantomData, sync::Arc};

pub struct AttachedImagesLoader<E: FormResponse> {
  db: ConnectionWrapper,
  scope: Select<active_storage_attachments::Entity>,
  _phantom: PhantomData<E>,
}

impl<E: FormResponse> AttachedImagesLoader<E> {
  pub fn new(db: ConnectionWrapper) -> Self {
    AttachedImagesLoader {
      db,
      scope: E::attached_images_scope(),
      _phantom: PhantomData,
    }
  }
}

#[async_trait::async_trait]
impl<E: FormResponse + Clone + 'static> Loader<i64> for AttachedImagesLoader<E> {
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
            let attachments = acc
              .entry(attachment.record_id)
              .or_insert_with(Default::default);
            if let Some(blob) = blob {
              attachments.push(blob);
            }
            acc
          },
        ),
    )
  }
}
