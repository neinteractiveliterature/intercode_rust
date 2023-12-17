use async_graphql::*;
use intercode_entities::form_sections;
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, loader_result_to_required_single, model_backed_type,
};

use super::{FormFormsFields, FormItemFormsFields};

model_backed_type!(FormSectionFormsFields, form_sections::Model);

impl FormSectionFormsFields {
  pub async fn form(&self, ctx: &Context<'_>) -> Result<FormFormsFields, Error> {
    let loader_result = load_one_by_model_id!(form_section_form, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      FormFormsFields
    ))
  }

  pub async fn form_items(&self, ctx: &Context<'_>) -> Result<Vec<FormItemFormsFields>, Error> {
    let loader_result = load_one_by_model_id!(form_section_form_items, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, FormItemFormsFields))
  }
}

#[Object]
impl FormSectionFormsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn position(&self) -> i32 {
    self.model.position
  }

  async fn title(&self) -> Option<&str> {
    self.model.title.as_deref()
  }
}
