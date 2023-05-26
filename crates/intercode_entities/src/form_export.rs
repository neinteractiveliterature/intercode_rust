use itertools::Itertools;
use sea_orm::JsonValue;
use serde::{Deserialize, Serialize};

use crate::{form_items, form_sections, forms, model_ext::form_item_permissions::FormItemRole};

#[derive(Serialize, Deserialize)]
pub struct FormItemExport {
  pub item_type: Option<String>,
  pub identifier: Option<String>,
  pub admin_description: Option<String>,
  pub public_description: Option<String>,
  pub default_value: Option<JsonValue>,
  pub visibility: FormItemRole,
  pub writeability: FormItemRole,
  #[serde(flatten)]
  pub properties: JsonValue,
}

impl From<form_items::Model> for FormItemExport {
  fn from(value: form_items::Model) -> Self {
    FormItemExport {
      item_type: value.item_type,
      identifier: value.identifier,
      admin_description: value.admin_description,
      public_description: value.public_description,
      default_value: value.default_value,
      visibility: value.visibility,
      writeability: value.writeability,
      properties: value.properties.unwrap_or_default(),
    }
  }
}

impl From<FormItemExport> for form_items::Model {
  fn from(value: FormItemExport) -> Self {
    form_items::Model {
      item_type: value.item_type,
      identifier: value.identifier,
      admin_description: value.admin_description,
      public_description: value.public_description,
      default_value: value.default_value,
      visibility: value.visibility,
      writeability: value.writeability,
      properties: Some(value.properties),
      ..Default::default()
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct FormSectionExport {
  pub title: Option<String>,
  pub section_items: Vec<FormItemExport>,
}

#[derive(Serialize, Deserialize)]
pub struct FormExport {
  pub title: Option<String>,
  pub form_type: String,
  pub sections: Vec<FormSectionExport>,
}

impl FormExport {
  pub fn from_form(
    form: &forms::Model,
    mut form_sections: Vec<&form_sections::Model>,
    form_items: Vec<&form_items::Model>,
  ) -> FormExport {
    let mut form_items_by_section_id = form_items
      .iter()
      .map(|item| (item.form_section_id, *item))
      .into_group_map();

    form_sections.sort_by_key(|section| section.position);
    let export_sections = form_sections
      .into_iter()
      .map(|section| {
        let mut items = form_items_by_section_id
          .get_mut(&Some(section.id))
          .cloned()
          .unwrap_or_default();
        items.sort_by_key(|item| item.position);

        FormSectionExport {
          title: section.title.clone(),
          section_items: items
            .into_iter()
            .map(|item| FormItemExport::from(item.clone()))
            .collect_vec(),
        }
      })
      .collect::<Vec<_>>();

    FormExport {
      title: form.title.clone(),
      form_type: form.form_type.clone(),
      sections: export_sections,
    }
  }
}
