use async_graphql::{Context, Error, Object, ID};
use intercode_entities::event_categories;
use intercode_inflector::inflector::string::pluralize;
use seawater::loaders::ExpectModels;

use crate::{model_backed_type, QueryData};

use super::{FormType, ModelBackedType};

model_backed_type!(EventCategoryType, event_categories::Model);

#[Object(name = "EventCategory")]
impl EventCategoryType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "can_provide_tickets")]
  async fn can_provide_tickets(&self) -> bool {
    self.model.can_provide_tickets
  }

  #[graphql(name = "default_color")]
  async fn default_color(&self) -> &str {
    &self.model.default_color
  }

  #[graphql(name = "event_form")]
  pub async fn event_form(&self, ctx: &Context<'_>) -> Result<FormType, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(FormType::new(
      query_data
        .loaders
        .event_category_event_form
        .load_one(self.model.id)
        .await?
        .expect_one()?
        .clone(),
    ))
  }

  #[graphql(name = "event_proposal_form")]
  async fn event_proposal_form(&self, ctx: &Context<'_>) -> Result<Option<FormType>, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      query_data
        .loaders
        .event_category_event_proposal_form
        .load_one(self.model.id)
        .await?
        .try_one()
        .map(|model| FormType::new(model.clone())),
    )
  }

  #[graphql(name = "full_color")]
  async fn full_color(&self) -> &str {
    &self.model.full_color
  }

  async fn name(&self) -> &str {
    &self.model.name
  }

  #[graphql(name = "scheduling_ui")]
  async fn scheduling_ui(&self) -> &str {
    &self.model.scheduling_ui
  }

  #[graphql(name = "signed_up_color")]
  async fn signed_up_color(&self) -> &str {
    &self.model.signed_up_color
  }

  #[graphql(name = "team_member_name")]
  async fn team_member_name(&self) -> &str {
    &self.model.team_member_name
  }

  async fn team_member_name_plural(&self) -> String {
    pluralize::to_plural(&self.model.team_member_name)
  }
}
