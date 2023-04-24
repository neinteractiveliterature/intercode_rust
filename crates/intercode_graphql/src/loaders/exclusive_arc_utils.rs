use std::{collections::HashMap, hash::Hash};

#[macro_export(crate)]
macro_rules! exclusive_arc_variant_loader {
  ($name: ident, $entity: path, $ref_enum: path, $ref_variant: path, $model_enum: path, $model_variant: path) => {
    async fn $name(
      keys: &[$ref_enum],
      loader: &DataLoader<EntityRelationLoader<permissions::Entity, $entity>>,
    ) -> Result<HashMap<$ref_enum, $model_enum>, Arc<DbErr>> {
      let ids = keys
        .iter()
        .filter_map(|key| {
          if let $ref_variant(id) = key {
            Some(id)
          } else {
            None
          }
        })
        .copied()
        .collect::<Vec<_>>();

      if ids.is_empty() {
        return Ok(HashMap::new());
      }

      let results = loader.load_many(ids).await?;

      Ok(
        results
          .into_iter()
          .filter_map(|(id, result)| {
            result
              .try_one()
              .map(|model| ($ref_variant(id), $model_variant(model.clone())))
          })
          .collect(),
      )
    }
  };
}

pub fn merge_hash_maps<K: Eq + Hash, V>(hash_maps: Vec<HashMap<K, V>>) -> HashMap<K, V> {
  let total_capacity = hash_maps
    .iter()
    .fold(0, |acc, hash_map| acc + hash_map.len());

  hash_maps.into_iter().fold(
    HashMap::with_capacity(total_capacity),
    |mut acc, hash_map| {
      acc.extend(hash_map.into_iter());
      acc
    },
  )
}
