use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{forms, FormExport};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, model_backed_type, scalars::JsonScalar,
};
use intercode_graphql_loaders::LoaderManager;
use seawater::loaders::ExpectModels;

use crate::objects::FormSectionType;

model_backed_type!(FormFormsFields, forms::Model);

#[Object]
impl FormFormsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "export_json")]
  async fn export_json(&self, ctx: &Context<'_>) -> Result<JsonScalar> {
    let section_loader_result = load_one_by_model_id!(form_form_sections, ctx, self)?;
    let sections = section_loader_result.expect_models()?;
    let item_loader_results = ctx
      .data::<Arc<LoaderManager>>()?
      .form_section_form_items()
      .load_many(sections.iter().map(|section| section.id))
      .await?;
    let items = item_loader_results
      .into_values()
      .flat_map(|loader_result| loader_result.models)
      .collect::<Vec<_>>();
    let export = FormExport::from_form(
      &self.model,
      sections.iter().collect(),
      items.iter().collect(),
    );
    let json = serde_json::to_value(export)?;
    Ok(JsonScalar(json))
  }

  #[graphql(name = "form_sections")]
  async fn form_sections(&self, ctx: &Context<'_>) -> Result<Vec<FormSectionType>, Error> {
    let loader_result = load_one_by_model_id!(form_form_sections, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, FormSectionType))
  }

  #[graphql(name = "form_type")]
  async fn form_type(&self) -> &str {
    &self.model.form_type
  }

  async fn title(&self) -> Option<&str> {
    self.model.title.as_deref()
  }
}