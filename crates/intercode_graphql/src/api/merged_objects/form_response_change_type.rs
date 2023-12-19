use async_graphql::{Context, Object, Result};
use intercode_entities::form_response_changes;
use intercode_forms::partial_objects::{
  FormResponseChangeFormsExtensions, FormResponseChangeFormsFields,
};
use intercode_graphql_core::model_backed_type;

use crate::merged_model_backed_type;

use super::UserConProfileType;

model_backed_type!(FormResponseChangeGlueFields, form_response_changes::Model);

impl FormResponseChangeFormsExtensions for FormResponseChangeGlueFields {}

#[Object]
impl FormResponseChangeGlueFields {
  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    FormResponseChangeFormsExtensions::user_con_profile(self, ctx).await
  }
}

merged_model_backed_type!(
  FormResponseChangeType,
  form_response_changes::Model,
  "FormResponseChange",
  FormResponseChangeFormsFields,
  FormResponseChangeGlueFields
);
