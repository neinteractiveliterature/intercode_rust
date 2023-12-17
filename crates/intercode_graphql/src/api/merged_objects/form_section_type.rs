use async_graphql::{Context, Error, Object};
use intercode_entities::form_sections;
use intercode_forms::partial_objects::FormSectionFormsFields;
use intercode_graphql_core::{model_backed_type, ModelBackedType};

use crate::merged_model_backed_type;

use super::{FormItemType, FormType};

model_backed_type!(FormSectionGlueFields, form_sections::Model);

#[Object]
impl FormSectionGlueFields {
  pub async fn form(&self, ctx: &Context<'_>) -> Result<FormType, Error> {
    FormSectionFormsFields::from_type(self.clone())
      .form(ctx)
      .await
      .map(FormType::from_type)
  }

  #[graphql(name = "form_items")]
  pub async fn form_items(&self, ctx: &Context<'_>) -> Result<Vec<FormItemType>, Error> {
    FormSectionFormsFields::from_type(self.clone())
      .form_items(ctx)
      .await
      .map(|res| res.into_iter().map(FormItemType::from_type).collect())
  }
}

merged_model_backed_type!(
  FormSectionType,
  form_sections::Model,
  "FormSection",
  FormSectionGlueFields,
  FormSectionFormsFields
);
