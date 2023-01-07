use async_graphql::{Context, Error, Object};
use chrono::{Duration, NaiveDateTime};
use intercode_entities::runs;
use seawater::loaders::ExpectModels;

use crate::{model_backed_type, QueryData};

use super::{ModelBackedType, RoomType};

model_backed_type!(RunType, runs::Model);

#[Object(name = "Run")]
impl RunType {
  async fn id(&self) -> i64 {
    self.model.id
  }

  #[graphql(name = "ends_at")]
  async fn ends_at(&self, ctx: &Context<'_>) -> Result<Option<NaiveDateTime>, Error> {
    let starts_at = self.model.starts_at;

    if let Some(starts_at) = starts_at {
      let query_data = ctx.data::<QueryData>()?;
      let length_seconds = query_data
        .loaders
        .run_event
        .load_one(self.model.id)
        .await?
        .expect_one()?
        .length_seconds;

      Ok(Some(starts_at + Duration::seconds(length_seconds.into())))
    } else {
      Ok(None)
    }
  }

  #[graphql(name = "room_names")]
  async fn room_names(&self, ctx: &Context<'_>) -> Result<Vec<Option<String>>, Error> {
    Ok(
      self
        .rooms(ctx)
        .await?
        .into_iter()
        .map(|room| room.get_model().name.clone())
        .collect(),
    )
  }

  async fn rooms(&self, ctx: &Context<'_>) -> Result<Vec<RoomType>, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      query_data
        .loaders
        .run_rooms
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .map(|model| RoomType::new(model.clone()))
        .collect(),
    )
  }

  #[graphql(name = "schedule_note")]
  async fn schedule_note(&self) -> Option<&str> {
    self.model.schedule_note.as_deref()
  }

  #[graphql(name = "starts_at")]
  async fn starts_at(&self) -> Option<NaiveDateTime> {
    self.model.starts_at
  }

  #[graphql(name = "title_suffix")]
  async fn title_suffix(&self) -> Option<&str> {
    self.model.title_suffix.as_deref()
  }
}
