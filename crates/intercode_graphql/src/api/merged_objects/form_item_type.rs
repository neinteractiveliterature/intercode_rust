use async_graphql::{Context, Error, Object};
use intercode_entities::form_items;
use intercode_forms::partial_objects::FormItemFormsFields;
use intercode_graphql_core::{model_backed_type, ModelBackedType};

use crate::merged_model_backed_type;

use super::FormSectionType;

model_backed_type!(FormItemGlueFields, form_items::Model);

#[Object]
impl FormItemGlueFields {
  #[graphql(name = "form_section")]
  pub async fn form_section(&self, ctx: &Context<'_>) -> Result<FormSectionType, Error> {
    FormItemFormsFields::from_type(self.clone())
      .form_section(ctx)
      .await
      .map(FormSectionType::from_type)
  }
}

merged_model_backed_type!(
  FormItemType,
  form_items::Model,
  "FormItem",
  FormItemGlueFields,
  FormItemFormsFields
);
