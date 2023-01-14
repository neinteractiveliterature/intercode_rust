use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;
use intercode_entities::{
  active_storage_blobs, form_items,
  model_ext::{form_item_permissions::FormItemRole, FormResponse},
};
use intercode_liquid::render_markdown;
use sea_orm::{ConnectionTrait, DbErr};
use serde_json::Value;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum FormResponsePresentationFormat {
  Plain,
  Markdown,
}

pub fn form_response_as_json<'a>(
  form_response: &dyn FormResponse,
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
                &form_item,
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

pub async fn attached_images_by_filename<C: ConnectionTrait>(
  form_response: &dyn FormResponse,
  db: &C,
) -> Result<HashMap<String, active_storage_blobs::Model>, DbErr> {
  Ok(
    form_response
      .attached_images()
      .find_also_related(active_storage_blobs::Entity)
      .all(db)
      .await?
      .into_iter()
      .filter_map(|(_image, blob)| blob.map(|blob| (blob.filename.clone(), blob)))
      .collect(),
  )
}

fn form_item_is_markdown_format(form_item: &form_items::Model) -> bool {
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
    if form_item_is_markdown_format(form_item) {
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
    FormResponsePresentationFormat::Markdown if form_item_is_markdown_format(form_item) => {
      Some(Value::String(format!("<em>{}</em>", hidden_text)))
    }
    _ => Some(Value::String(hidden_text)),
  }
}
