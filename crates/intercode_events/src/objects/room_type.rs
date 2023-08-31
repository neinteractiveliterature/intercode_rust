use async_graphql::*;
use intercode_entities::rooms;
use intercode_graphql_core::{load_one_by_model_id, loader_result_to_many, model_backed_type};
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};

use crate::{partial_objects::RunEventsFields, policies::RoomPolicy};

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

  async fn runs(&self, ctx: &Context<'_>) -> Result<Vec<RunEventsFields>, Error> {
    let loader_result = load_one_by_model_id!(room_runs, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, RunEventsFields))
  }
}
