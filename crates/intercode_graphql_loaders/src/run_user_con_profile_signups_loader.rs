use async_graphql::dataloader::Loader;
use async_trait::async_trait;
use intercode_entities::signups;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use seawater::ConnectionWrapper;
use std::{collections::HashMap, sync::Arc};

pub struct RunUserConProfileSignupsLoader {
  db: ConnectionWrapper,
  user_con_profile_id: i64,
}

impl RunUserConProfileSignupsLoader {
  pub fn new(db: ConnectionWrapper, user_con_profile_id: i64) -> Self {
    RunUserConProfileSignupsLoader {
      db,
      user_con_profile_id,
    }
  }
}

#[async_trait]
impl Loader<i64> for RunUserConProfileSignupsLoader {
  type Value = Vec<signups::Model>;
  type Error = Arc<DbErr>;

  async fn load(
    &self,
    keys: &[i64],
  ) -> Result<std::collections::HashMap<i64, Self::Value>, Self::Error> {
    Ok(
      signups::Entity::find()
        .filter(signups::Column::RunId.is_in(keys.iter().copied()))
        .filter(signups::Column::UserConProfileId.eq(self.user_con_profile_id))
        .all(&self.db)
        .await?
        .into_iter()
        .fold(
          HashMap::<i64, Self::Value>::with_capacity(keys.len()),
          |mut acc, signup| {
            let signups = acc.entry(signup.run_id).or_insert_with(Default::default);
            signups.push(signup);
            acc
          },
        ),
    )
  }
}
