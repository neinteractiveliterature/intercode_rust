use async_graphql::{Context, Error, Object};
use chrono::{Duration, NaiveDateTime};
use intercode_entities::runs;

use crate::{loaders::expect::ExpectModels, model_backed_type, SchemaData};

use super::{ModelBackedType, RoomType};

model_backed_type!(RunType, runs::Model);

#[Object]
impl RunType {
  async fn id(&self) -> i64 {
    self.model.id
  }

  #[graphql(name = "ends_at")]
  async fn ends_at(&self, ctx: &Context<'_>) -> Result<Option<NaiveDateTime>, Error> {
    let starts_at = self.model.starts_at;

    if let Some(starts_at) = starts_at {
      let schema_data = ctx.data::<SchemaData>()?;
      let length_seconds = schema_data
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

  async fn rooms(&self, ctx: &Context<'_>) -> Result<Vec<RoomType>, Error> {
    let schema_data = ctx.data::<SchemaData>()?;

    Ok(
      schema_data
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

  #[graphql(name = "starts_at")]
  async fn starts_at(&self) -> Option<NaiveDateTime> {
    self.model.starts_at
  }
}
