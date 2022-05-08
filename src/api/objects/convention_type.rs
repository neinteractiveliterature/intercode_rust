use crate::conventions;
use async_graphql::*;

pub struct ConventionType {
  model: conventions::Model,
}

impl ConventionType {
  pub fn new(model: conventions::Model) -> ConventionType {
    ConventionType { model }
  }
}

#[Object]
impl ConventionType {
  async fn id(&self) -> ID {
    ID(self.model.id.to_string())
  }

  async fn name(&self) -> &Option<String> {
    &self.model.name
  }
}
