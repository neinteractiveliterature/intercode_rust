use async_graphql::*;
use intercode_entities::forms;
use seawater::loaders::ExpectModels;

use crate::{model_backed_type, QueryData};

use super::{FormSectionType, ModelBackedType};
model_backed_type!(FormType, forms::Model);

#[Object(name = "Form")]
impl FormType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "form_sections")]
  async fn form_sections(&self, ctx: &Context<'_>) -> Result<Vec<FormSectionType>, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      query_data
        .loaders
        .form_form_sections
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .cloned()
        .map(FormSectionType::new)
        .collect(),
    )
  }

  #[graphql(name = "form_type")]
  async fn form_type(&self) -> &str {
    &self.model.form_type
  }

  async fn title(&self) -> Option<&str> {
    self.model.title.as_deref()
  }
}
