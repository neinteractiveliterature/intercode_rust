use std::collections::HashSet;

use async_graphql::{Context, Error, Interface};
use async_trait::async_trait;
use intercode_entities::model_ext::form_item_permissions::FormItemRole;
use intercode_entities::model_ext::FormResponse;
use intercode_entities::{form_items, forms};
use intercode_inflector::IntercodeInflector;
use seawater::loaders::ExpectModels;

use crate::api::objects::{EventType, ModelBackedType};
use crate::api::scalars::JsonScalar;
use crate::presenters::form_response_presenter::{
  attached_images_by_filename, form_response_as_json, FormResponsePresentationFormat,
};
use crate::{QueryData, SchemaData};

async fn load_filtered_form_items(
  query_data: &QueryData,
  form_id: i64,
  item_identifiers: Option<Vec<String>>,
) -> Result<Vec<form_items::Model>, Error> {
  let form_items_result = query_data
    .loaders()
    .form_form_items()
    .load_one(form_id)
    .await?;
  let form_items = form_items_result.expect_models()?;
  let form_items: Vec<form_items::Model> = match item_identifiers {
    Some(item_identifiers) => {
      let item_identifiers: HashSet<String> = HashSet::from_iter(item_identifiers.into_iter());
      form_items
        .iter()
        .filter(|item| {
          item
            .identifier
            .as_ref()
            .map(|identifier| item_identifiers.contains(identifier))
            .unwrap_or(false)
        })
        .cloned()
        .collect()
    }
    None => form_items.to_vec(),
  };

  Ok(form_items)
}

#[derive(Interface)]
#[graphql(field(
  name = "form_response_attrs_json_with_rendered_markdown",
  type = "JsonScalar",
  arg(name = "itemIdentifiers", type = "Option<Vec<String>>")
))]
pub enum FormResponseInterface {
  Event(EventType),
}

#[async_trait]
pub trait FormResponseImplementation<M>
where
  Self: ModelBackedType<Model = M>,
  M: sea_orm::ModelTrait + FormResponse + Send + Sync,
{
  async fn get_form(&self, ctx: &Context<'_>) -> Result<forms::Model, Error>;
  async fn get_team_member_name(&self, ctx: &Context<'_>) -> Result<String, Error>;
  async fn get_viewer_role(&self, ctx: &Context<'_>) -> Result<FormItemRole, Error>;
  async fn get_writer_role(&self, ctx: &Context<'_>) -> Result<FormItemRole, Error>;

  async fn form_response_attrs_json(
    &self,
    ctx: &Context<'_>,
    item_identifiers: Option<Vec<String>>,
  ) -> Result<JsonScalar, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    let query_data = ctx.data::<QueryData>()?;
    let form = self.get_form(ctx).await?;

    let model = self.get_model();
    let attached_images = attached_images_by_filename(model, query_data).await?;

    let viewer_role = self.get_viewer_role(ctx).await?;

    let form_items = load_filtered_form_items(query_data, form.id, item_identifiers).await?;

    Ok(JsonScalar(form_response_as_json(
      model,
      form_items.iter(),
      &attached_images,
      viewer_role,
      FormResponsePresentationFormat::Plain,
      &schema_data.language_loader,
      &IntercodeInflector::new().pluralize(&self.get_team_member_name(ctx).await?),
    )))
  }

  async fn form_response_attrs_json_with_rendered_markdown(
    &self,
    ctx: &Context<'_>,
    item_identifiers: Option<Vec<String>>,
  ) -> Result<JsonScalar, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    let query_data = ctx.data::<QueryData>()?;
    let form = self.get_form(ctx).await?;

    let model = self.get_model();
    let attached_images = attached_images_by_filename(model, query_data).await?;

    let viewer_role = self.get_viewer_role(ctx).await?;

    let form_items = load_filtered_form_items(query_data, form.id, item_identifiers).await?;

    Ok(JsonScalar(form_response_as_json(
      model,
      form_items.iter(),
      &attached_images,
      viewer_role,
      FormResponsePresentationFormat::Html,
      &schema_data.language_loader,
      &IntercodeInflector::new().pluralize(&self.get_team_member_name(ctx).await?),
    )))
  }
}
