use async_graphql::Object;
use intercode_entities::event_categories;

use crate::model_backed_type;

model_backed_type!(EventCategoryType, event_categories::Model);

#[Object(name = "EventCategory")]
impl EventCategoryType {
  async fn id(&self) -> i64 {
    self.model.id
  }

  async fn name(&self) -> &str {
    &self.model.name
  }

  #[graphql(name = "team_member_name")]
  async fn team_member_name(&self) -> &str {
    &self.model.team_member_name
  }
}
