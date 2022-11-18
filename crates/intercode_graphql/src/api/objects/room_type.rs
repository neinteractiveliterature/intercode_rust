use async_graphql::*;
use intercode_entities::rooms;
use intercode_policies::{policies::RoomPolicy, ReadManageAction};
use seawater::loaders::ExpectModels;

use crate::{model_backed_type, QueryData};

use super::{ModelBackedType, RunType};
model_backed_type!(RoomType, rooms::Model);

#[Object(
  name = "Room",
  guard = "self.policy_guard::<RoomPolicy>(ReadManageAction::Read)"
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
        .data::<QueryData>()?
        .loaders
        .room_runs
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .map(|run| RunType::new(run.clone()))
        .collect(),
    )
  }
}
