use async_graphql::dataloader::Loader;
use async_session::async_trait;
use intercode_entities::signups;
use sea_orm::{
  ColumnTrait, DbErr, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect,
};
use seawater::ConnectionWrapper;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Hash, Debug, PartialEq, Eq, FromQueryResult)]
pub struct WaitlistPositionLoaderKey {
  signup_id: i64,
  run_id: i64,
}

impl From<signups::Model> for WaitlistPositionLoaderKey {
  fn from(value: signups::Model) -> Self {
    WaitlistPositionLoaderKey {
      signup_id: value.id,
      run_id: value.run_id,
    }
  }
}

pub struct WaitlistPositionLoader {
  db: ConnectionWrapper,
}

impl WaitlistPositionLoader {
  pub fn new(db: ConnectionWrapper) -> Self {
    WaitlistPositionLoader { db }
  }
}

#[async_trait]
impl Loader<WaitlistPositionLoaderKey> for WaitlistPositionLoader {
  type Value = Option<usize>;
  type Error = Arc<DbErr>;

  async fn load(
    &self,
    keys: &[WaitlistPositionLoaderKey],
  ) -> Result<std::collections::HashMap<WaitlistPositionLoaderKey, Self::Value>, Self::Error> {
    let signup_ids_by_run_id_ordered: HashMap<i64, Vec<i64>> = signups::Entity::find()
      .filter(signups::Column::RunId.is_in(keys.iter().map(|key| key.run_id)))
      .filter(signups::Column::State.eq("waitlisted"))
      .select_only()
      .column_as(signups::Column::Id, "signup_id")
      .column(signups::Column::RunId)
      .order_by_asc(signups::Column::RunId)
      .order_by_asc(signups::Column::CreatedAt)
      .into_model::<WaitlistPositionLoaderKey>()
      .all(&self.db)
      .await
      .map_err(Arc::new)?
      .iter()
      .fold(HashMap::new(), |mut acc, row| {
        let signup_ids = acc.entry(row.run_id).or_default();
        signup_ids.push(row.signup_id);
        acc
      });

    Ok(keys.iter().fold(HashMap::new(), |mut acc, key| {
      let position = signup_ids_by_run_id_ordered
        .get(&key.run_id)
        .and_then(|signup_ids| {
          signup_ids
            .iter()
            .position(|signup_id| *signup_id == key.signup_id)
        })
        .map(|pos| pos + 1);
      acc.insert(key.clone(), position);
      acc
    }))
  }
}
