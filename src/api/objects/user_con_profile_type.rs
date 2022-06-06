use crate::{user_con_profiles, SchemaData};
use async_graphql::*;

use super::{ConventionType, ModelBackedType};
pub struct UserConProfileType {
  model: user_con_profiles::Model,
}

impl ModelBackedType<user_con_profiles::Model> for UserConProfileType {
  fn new(model: user_con_profiles::Model) -> Self {
    UserConProfileType { model }
  }
}

#[Object]
impl UserConProfileType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn convention(&self, ctx: &Context<'_>) -> Result<Option<ConventionType>, Error> {
    let loader = &ctx.data::<SchemaData>()?.convention_id_loader;

    let model = loader.load_one(self.model.convention_id).await?;
    if let Some(model) = model {
      Ok(Some(ConventionType::new(model)))
    } else {
      Err(Error::new(format!(
        "{} {} not found",
        <ConventionType as async_graphql::OutputType>::type_name(),
        self.model.convention_id
      )))
    }
  }
}
