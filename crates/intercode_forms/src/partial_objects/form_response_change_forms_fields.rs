use async_graphql::{Context, Object, Result};
use async_trait::async_trait;
use intercode_entities::{form_response_changes, user_con_profiles};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_required_single, model_backed_type,
  scalars::{DateScalar, JsonScalar},
  ModelBackedType,
};

#[async_trait]
pub trait FormResponseChangeFormsExtensions
where
  Self: ModelBackedType<Model = form_response_changes::Model>,
{
  async fn user_con_profile<T: ModelBackedType<Model = user_con_profiles::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<T> {
    let loader_result = load_one_by_model_id!(form_response_change_user_con_profile, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }
}

model_backed_type!(FormResponseChangeFormsFields, form_response_changes::Model);

#[Object]
impl FormResponseChangeFormsFields {
  async fn compacted(&self) -> bool {
    self.model.compacted
  }

  #[graphql(name = "created_at")]
  async fn created_at(&self) -> Result<DateScalar> {
    self.model.created_at.try_into()
  }

  #[graphql(name = "field_identifier")]
  async fn field_identifier(&self) -> &str {
    &self.model.field_identifier
  }

  #[graphql(name = "new_value")]
  async fn new_value(&self) -> Option<JsonScalar> {
    self.model.new_value.clone().map(JsonScalar)
  }

  #[graphql(name = "notified_at")]
  async fn notified_at(&self) -> Result<Option<DateScalar>> {
    self.model.notified_at.map(DateScalar::try_from).transpose()
  }

  #[graphql(name = "previous_value")]
  async fn previous_value(&self) -> Option<JsonScalar> {
    self.model.previous_value.clone().map(JsonScalar)
  }

  #[graphql(name = "updated_at")]
  async fn updated_at(&self) -> Result<DateScalar> {
    self.model.updated_at.try_into()
  }
}
