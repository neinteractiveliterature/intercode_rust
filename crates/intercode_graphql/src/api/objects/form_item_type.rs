use async_graphql::*;
use intercode_entities::{form_items, model_ext::form_item_permissions::FormItemRole};
use intercode_liquid::render_markdown;

use crate::{api::scalars::JsonScalar, model_backed_type};
model_backed_type!(FormItemType, form_items::Model);

#[Object(name = "FormItem")]
impl FormItemType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "admin_description")]
  async fn admin_description(&self) -> Option<&str> {
    self.model.admin_description.as_deref()
  }

  #[graphql(name = "default_value")]
  async fn default_value(&self) -> Result<Option<String>, Error> {
    self
      .model
      .default_value
      .as_ref()
      .map(serde_json::to_string)
      .transpose()
      .map_err(|err| err.into())
  }

  #[graphql(name = "expose_in")]
  async fn expose_in(&self) -> &Option<Vec<String>> {
    &self.model.expose_in
  }

  async fn identifier(&self) -> Option<&str> {
    self.model.identifier.as_deref()
  }

  #[graphql(name = "item_type")]
  async fn item_type(&self) -> Option<&str> {
    self.model.item_type.as_deref()
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
  async fn rendered_properties(&self) -> Result<Option<String>, Error> {
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

        return Ok(Some(serde_json::to_string(&rendered)?));
      }
    }

    Ok(None)
  }

  async fn visibility(&self) -> FormItemRole {
    self.model.visibility
  }

  async fn writeability(&self) -> FormItemRole {
    self.model.writeability
  }
}
