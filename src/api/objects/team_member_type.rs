use crate::team_members;
use async_graphql::*;

use crate::model_backed_type;
model_backed_type!(TeamMemberType, team_members::Model);

#[Object]
impl TeamMemberType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }
}
