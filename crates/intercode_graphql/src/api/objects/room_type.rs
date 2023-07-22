use std::sync::Arc;

use async_graphql::*;
use intercode_entities::rooms;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{policies::RoomPolicy, ModelBackedTypeGuardablePolicy, ReadManageAction};
use seawater::loaders::ExpectModels;

use super::RunType;
model_backed_type!(RoomType, rooms::Model);

#[Object(
  name = "Room",
  guard = "RoomPolicy::model_guard(ReadManageAction::Read, self)"
)]
impl RoomType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }

  async fn runs(&self, ctx: &Context<'_>) -> Result<Vec<RunType>, Error> {
    Ok(
      ctx
        .data::<Arc<LoaderManager>>()?
        .room_runs()
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .map(|run| RunType::new(run.clone()))
        .collect(),
    )
  }
}
