use async_graphql::dataloader::Loader;
use async_session::async_trait;
use intercode_entities::signup_requests;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use seawater::ConnectionWrapper;
use std::{collections::HashMap, sync::Arc};

pub struct RunUserConProfileSignupRequestsLoader {
  db: ConnectionWrapper,
  user_con_profile_id: i64,
}

impl RunUserConProfileSignupRequestsLoader {
  pub fn new(db: ConnectionWrapper, user_con_profile_id: i64) -> Self {
    RunUserConProfileSignupRequestsLoader {
      db,
      user_con_profile_id,
    }
  }
}

#[async_trait]
impl Loader<i64> for RunUserConProfileSignupRequestsLoader {
  type Value = Vec<signup_requests::Model>;
  type Error = Arc<DbErr>;

  async fn load(
    &self,
    keys: &[i64],
  ) -> Result<std::collections::HashMap<i64, Self::Value>, Self::Error> {
    Ok(
      signup_requests::Entity::find()
        .filter(signup_requests::Column::TargetRunId.is_in(keys.iter().copied()))
        .filter(signup_requests::Column::UserConProfileId.eq(self.user_con_profile_id))
        .all(&self.db)
        .await?
        .into_iter()
        .fold(
          HashMap::<i64, Self::Value>::with_capacity(keys.len()),
          |mut acc, signup_request| {
            let signup_requests = acc
              .entry(signup_request.target_run_id)
              .or_insert_with(Default::default);
            signup_requests.push(signup_request);
            acc
          },
        ),
    )
  }
}
