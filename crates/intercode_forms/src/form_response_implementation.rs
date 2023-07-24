use std::{
  collections::{HashMap, HashSet},
  sync::Arc,
};

use async_graphql::{Context, Error};
use async_trait::async_trait;
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;
use intercode_entities::{
  active_storage_blobs, form_items, forms,
  model_ext::{form_item_permissions::FormItemRole, FormResponse},
};
use intercode_graphql_core::{scalars::JsonScalar, schema_data::SchemaData, ModelBackedType};
use intercode_graphql_loaders::{
  attached_images_by_filename::attached_images_by_filename, LoaderManager,
};
use intercode_inflector::IntercodeInflector;
use intercode_liquid::render_markdown;
use sea_orm::EntityTrait;
use seawater::loaders::ExpectModels;
use serde_json::Value;

async fn load_filtered_form_items(
  loaders: &LoaderManager,
  form_id: i64,
  item_identifiers: Option<Vec<String>>,
) -> Result<Vec<form_items::Model>, Error> {
  let form_items_result = loaders.form_form_items().load_one(form_id).await?;
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum FormResponsePresentationFormat {
  Plain,
  Html,
}

pub fn form_response_as_json<'a, E: EntityTrait>(
  form_response: &dyn FormResponse<Entity = E>,
  form_items: impl Iterator<Item = &'a form_items::Model>,
  attached_images: &HashMap<String, active_storage_blobs::Model>,
  viewer_role: FormItemRole,
  format: FormResponsePresentationFormat,
  language_loader: &FluentLanguageLoader,
  team_member_name_pluralized: &str,
) -> Value {
  Value::Object(
    form_items
      .filter_map(|form_item| {
        let identifier = &form_item.identifier;
        identifier.as_ref().and_then(|identifier| {
          let value = form_response.get(identifier);
          value.map(|value| {
            (
              identifier.clone(),
              render_form_item_response(
                form_item,
                &value,
                attached_images,
                viewer_role,
                format,
                language_loader,
                team_member_name_pluralized,
              ),
            )
          })
        })
      })
      .collect(),
  )
}

fn form_item_takes_markdown_input(form_item: &form_items::Model) -> bool {
  if matches!(&form_item.item_type, Some(item_type) if item_type == "free_text") {
    if let Some(Value::Object(properties)) = &form_item.properties {
      if matches!(
        properties.get("format"),
        Some(Value::String(format)) if format == "markdown"
      ) {
        return true;
      }
    }
  }
  false
}

pub fn render_form_item_response(
  form_item: &form_items::Model,
  value: &Value,
  attached_images: &HashMap<String, active_storage_blobs::Model>,
  viewer_role: FormItemRole,
  format: FormResponsePresentationFormat,
  language_loader: &FluentLanguageLoader,
  team_member_name_pluralized: &str,
) -> Value {
  if let Some(replacement_content) = replacement_content_for_form_item(
    form_item,
    value,
    viewer_role,
    format,
    language_loader,
    team_member_name_pluralized,
  ) {
    return replacement_content;
  }

  if let Value::String(value) = value {
    if form_item_takes_markdown_input(form_item) {
      return Value::String(render_markdown(value, attached_images));
    }

    Value::String(value.to_string())
  } else {
    value.to_owned()
  }
}

fn should_replace_content_for_form_item(
  form_item: &form_items::Model,
  value: &Value,
  viewer_role: FormItemRole,
) -> bool {
  viewer_role < form_item.visibility
    && !value.is_null()
    && !matches!(value, Value::String(text) if text.trim() == "")
}

fn replacement_content_for_form_item(
  form_item: &form_items::Model,
  value: &Value,
  viewer_role: FormItemRole,
  format: FormResponsePresentationFormat,
  language_loader: &FluentLanguageLoader,
  team_member_name_pluralized: &str,
) -> Option<Value> {
  if !should_replace_content_for_form_item(form_item, value, viewer_role) {
    return None;
  }

  let hidden_text = match viewer_role {
    FormItemRole::Normal => fl!(language_loader, "forms_hidden_text_normal"),
    FormItemRole::ConfirmedAttendee => fl!(language_loader, "forms_hidden_text_confirmed_attendee"),
    FormItemRole::TeamMember => fl!(
      language_loader,
      "forms_hidden_text_team_member",
      team_member_name_pluralized = team_member_name_pluralized
    ),
    FormItemRole::AllProfilesBasicAccess => fl!(
      language_loader,
      "forms_hidden_text_all_profiles_basic_access"
    ),
    FormItemRole::Admin => fl!(language_loader, "forms_hidden_text_admin"),
  };

  match format {
    FormResponsePresentationFormat::Html if form_item_takes_markdown_input(form_item) => {
      Some(Value::String(format!("<em>{}</em>", hidden_text)))
    }
    _ => Some(Value::String(hidden_text)),
  }
}

#[async_trait]
pub trait FormResponseImplementation<M>
where
  Self: ModelBackedType<Model = M>,
  M: sea_orm::ModelTrait + FormResponse + Send + Sync,
{
  async fn get_form(&self, ctx: &Context<'_>) -> Result<forms::Model, Error>;
  async fn get_team_member_name(&self, ctx: &Context<'_>) -> Result<String, Error>;

  async fn current_user_form_item_viewer_role(
    &self,
    ctx: &Context<'_>,
  ) -> Result<FormItemRole, Error>;

  async fn current_user_form_item_writer_role(
    &self,
    ctx: &Context<'_>,
  ) -> Result<FormItemRole, Error>;

  async fn form_response_attrs_json(
    &self,
    ctx: &Context<'_>,
    item_identifiers: Option<Vec<String>>,
  ) -> Result<JsonScalar, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let form = self.get_form(ctx).await?;

    let model = self.get_model();
    let attached_images = attached_images_by_filename(model, loaders).await?;

    let viewer_role = self.current_user_form_item_viewer_role(ctx).await?;

    let form_items = load_filtered_form_items(loaders, form.id, item_identifiers).await?;

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
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let form = self.get_form(ctx).await?;

    let model = self.get_model();
    let attached_images = attached_images_by_filename(model, loaders).await?;

    let viewer_role = self.current_user_form_item_viewer_role(ctx).await?;

    let form_items = load_filtered_form_items(loaders, form.id, item_identifiers).await?;

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
