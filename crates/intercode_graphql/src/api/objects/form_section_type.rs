use async_graphql::*;
use intercode_entities::form_sections;
use seawater::loaders::ExpectModels;

use crate::{model_backed_type, QueryData};

use super::{FormItemType, ModelBackedType};
model_backed_type!(FormSectionType, form_sections::Model);

#[Object(name = "FormSection")]
impl FormSectionType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "form_items")]
  async fn form_items(&self, ctx: &Context<'_>) -> Result<Vec<FormItemType>, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      query_data
        .loaders
        .form_section_form_items()
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .cloned()
        .map(FormItemType::new)
        .collect(),
    )
  }

  async fn position(&self) -> i32 {
    self.model.position
  }

  async fn title(&self) -> Option<&str> {
    self.model.title.as_deref()
  }
}
