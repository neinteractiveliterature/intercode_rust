use async_graphql::dataloader::Loader;
use async_session::async_trait;
use intercode_entities::event_ratings;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use seawater::ConnectionWrapper;
use std::sync::Arc;

pub struct EventUserConProfileEventRatingLoader {
  db: ConnectionWrapper,
  user_con_profile_id: i64,
}

impl EventUserConProfileEventRatingLoader {
  pub fn new(db: ConnectionWrapper, user_con_profile_id: i64) -> Self {
    EventUserConProfileEventRatingLoader {
      db,
      user_con_profile_id,
    }
  }
}

#[async_trait]
impl Loader<i64> for EventUserConProfileEventRatingLoader {
  type Value = event_ratings::Model;
  type Error = Arc<DbErr>;

  async fn load(
    &self,
    keys: &[i64],
  ) -> Result<std::collections::HashMap<i64, Self::Value>, Self::Error> {
    Ok(
      event_ratings::Entity::find()
        .filter(event_ratings::Column::EventId.is_in(keys.iter().copied()))
        .filter(event_ratings::Column::UserConProfileId.eq(self.user_con_profile_id))
        .all(&self.db)
        .await?
        .into_iter()
        .map(|event_rating| (event_rating.event_id, event_rating))
        .collect(),
    )
  }
}
