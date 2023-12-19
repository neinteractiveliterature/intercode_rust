use std::sync::Arc;

use async_graphql::*;
use async_trait::async_trait;
use intercode_entities::{form_sections, forms, FormExport};
use intercode_graphql_core::{
  enums::FormType, lax_id::LaxId, load_one_by_model_id, loader_result_to_many, model_backed_type,
  query_data::QueryData, scalars::JsonScalar, ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use sea_orm::{ColumnTrait, ModelTrait, QueryFilter};
use seawater::loaders::ExpectModels;

model_backed_type!(FormFormsFields, forms::Model);

#[async_trait]
pub trait FormFormsExtensions
where
  Self: ModelBackedType<Model = forms::Model>,
{
  async fn form_section<T: ModelBackedType<Model = form_sections::Model>>(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<T> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(T::new(
      self
        .get_model()
        .find_related(form_sections::Entity)
        .filter(form_sections::Column::Id.eq(LaxId::parse(id)?))
        .one(query_data.db())
        .await?
        .ok_or_else(|| Error::new("FormSection not found"))?,
    ))
  }

  async fn form_sections<T: ModelBackedType<Model = form_sections::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>, Error> {
    let loader_result = load_one_by_model_id!(form_form_sections, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }
}

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

  #[graphql(name = "form_type")]
  async fn form_type(&self) -> Result<FormType, Error> {
    self
      .model
      .form_type
      .as_str()
      .try_into()
      .map_err(Error::from)
  }

  async fn title(&self) -> &str {
    self.model.title.as_deref().unwrap_or_default()
  }
}
