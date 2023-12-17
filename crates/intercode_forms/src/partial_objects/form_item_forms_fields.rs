use async_graphql::*;
use intercode_entities::{form_items, model_ext::form_item_permissions::FormItemRole};
use intercode_graphql_core::{
  enums::FormItemExposeIn, load_one_by_model_id, loader_result_to_required_single,
  model_backed_type, scalars::JsonScalar,
};
use intercode_liquid::render_markdown;
use serde_json::json;

use super::FormSectionFormsFields;

model_backed_type!(FormItemFormsFields, form_items::Model);

impl FormItemFormsFields {
  pub async fn form_section(&self, ctx: &Context<'_>) -> Result<FormSectionFormsFields> {
    let loader_result = load_one_by_model_id!(form_item_form_section, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      FormSectionFormsFields
    ))
  }
}

#[Object]
impl FormItemFormsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "admin_description")]
  async fn admin_description(&self) -> Option<&str> {
    self.model.admin_description.as_deref()
  }

  #[graphql(name = "default_value")]
  async fn default_value(&self) -> Option<JsonScalar> {
    self.model.default_value.clone().map(JsonScalar)
  }

  #[graphql(name = "expose_in")]
  async fn expose_in(&self) -> Result<Option<Vec<FormItemExposeIn>>> {
    self
      .model
      .expose_in
      .as_ref()
      .map(|expose_in| {
        expose_in
          .iter()
          .map(|v| FormItemExposeIn::try_from(v.as_str()).map_err(Error::from))
          .collect::<Result<Vec<FormItemExposeIn>>>()
      })
      .transpose()
  }

  async fn identifier(&self) -> Option<&str> {
    self.model.identifier.as_deref()
  }

  #[graphql(name = "item_type")]
  async fn item_type(&self) -> &str {
    self.model.item_type.as_deref().unwrap_or_default()
  }

  async fn properties(&self) -> JsonScalar {
    JsonScalar(self.model.properties.clone().unwrap_or_default())
  }

  async fn position(&self) -> i32 {
    self.model.position
  }

  #[graphql(name = "public_description")]
  async fn public_description(&self) -> Option<&str> {
    self.model.public_description.as_deref()
  }

  #[graphql(name = "rendered_properties")]
  async fn rendered_properties(&self) -> Result<JsonScalar, Error> {
    if let Some(properties) = &self.model.properties {
      if let Some(properties) = properties.as_object() {
        let is_static_text = if let Some(item_type) = &self.model.item_type {
          item_type == "static_text"
        } else {
          false
        };

        let rendered = serde_json::Value::Object(
          properties
            .iter()
            .map(|(key, value)| {
              let value = if let Some(value) = value.as_str() {
                if (is_static_text && key == "content") || key == "caption" {
                  serde_json::Value::String(render_markdown(value, &Default::default()))
                } else {
                  serde_json::Value::String(value.to_string())
                }
              } else {
                value.clone()
              };

              (key.clone(), value)
            })
            .collect(),
        );

        return Ok(JsonScalar(rendered));
      }
    }

    Ok(JsonScalar(json!({})))
  }

  async fn visibility(&self) -> FormItemRole {
    self.model.visibility
  }

  async fn writeability(&self) -> FormItemRole {
    self.model.writeability
  }
}
