use async_graphql::{Context, Error, Object};
use intercode_entities::rooms;
use intercode_events::partial_objects::RoomEventsFields;
use intercode_graphql_core::{model_backed_type, ModelBackedType};

use crate::merged_model_backed_type;

use super::RunType;

model_backed_type!(RoomGlueFields, rooms::Model);

#[Object]
impl RoomGlueFields {
  pub async fn runs(&self, ctx: &Context<'_>) -> Result<Vec<RunType>, Error> {
    RoomEventsFields::from_type(self.clone())
      .runs(ctx)
      .await
      .map(|runs| runs.into_iter().map(RunType::from_type).collect())
  }
}

merged_model_backed_type!(
  RoomType,
  rooms::Model,
  "Room",
  RoomEventsFields,
  RoomGlueFields
);
