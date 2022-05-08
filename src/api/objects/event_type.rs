use crate::{events, SchemaData};
use async_graphql::*;

use super::{ConventionType, ModelBackedType};
pub struct EventType {
  model: events::Model,
}

impl ModelBackedType<events::Model> for EventType {
  fn new(model: events::Model) -> Self {
    EventType { model }
  }
}

#[Object]
impl EventType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn title(&self) -> &String {
    &self.model.title
  }

  async fn author(&self) -> &Option<String> {
    &self.model.author
  }

  async fn email(&self) -> &Option<String> {
    &self.model.email
  }

  async fn convention(&self, ctx: &Context<'_>) -> Result<Option<ConventionType>, Error> {
    let loader = &ctx.data::<SchemaData>()?.convention_id_loader;

    if let Some(convention_id) = self.model.convention_id {
      let model = loader.load_one(convention_id).await?;
      if let Some(model) = model {
        Ok(Some(ConventionType::new(model)))
      } else {
        Err(Error::new(format!(
          "{} {} not found",
          <ConventionType as async_graphql::OutputType>::type_name(),
          convention_id
        )))
      }
    } else {
      Ok(None)
    }
  }
}
