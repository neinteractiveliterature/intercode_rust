use std::collections::HashMap;

use async_graphql::Error;
use intercode_entities::{active_storage_blobs, model_ext::FormResponse};
use sea_orm::EntityTrait;

use crate::LoaderManager;

pub async fn attached_images_by_filename<E: EntityTrait>(
  form_response: &dyn FormResponse<Entity = E>,
  loaders: &LoaderManager,
) -> Result<HashMap<String, active_storage_blobs::Model>, Error> {
  Ok(
    loaders
      .event_attached_images
      .load_one(form_response.get_id())
      .await?
      .unwrap_or_default()
      .into_iter()
      .map(|blob| (blob.filename.clone(), blob))
      .collect(),
  )
}
