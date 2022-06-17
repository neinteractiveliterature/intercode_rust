use crate::{loaders::expect::ExpectModels, SchemaData};
use async_graphql::*;
use intercode_entities::team_members;

use crate::model_backed_type;

use super::{EventType, ModelBackedType};
model_backed_type!(TeamMemberType, team_members::Model);

#[Object]
impl TeamMemberType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn event(&self, ctx: &Context<'_>) -> Result<EventType, Error> {
    let loader = &ctx.data::<SchemaData>()?.loaders.team_member_event;

    let result = loader.load_one(self.model.id).await?;
    let event = result.expect_one()?;

    Ok(EventType::new(event.to_owned()))
  }
}
