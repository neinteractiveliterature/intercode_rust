use async_graphql::*;
use intercode_entities::forms;
use intercode_forms::partial_objects::FormFormsFields;
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, model_backed_type, ModelBackedType,
};

use super::{ConventionType, EventCategoryType};

model_backed_type!(FormGlueFields, forms::Model);

#[Object]
impl FormGlueFields {
  #[graphql(name = "event_categories")]
  async fn event_categories(&self, ctx: &Context<'_>) -> Result<Vec<EventCategoryType>, Error> {
    let loader_result = load_one_by_model_id!(form_event_categories, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, EventCategoryType))
  }

  #[graphql(name = "proposal_event_categories")]
  async fn proposal_event_categories(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<EventCategoryType>, Error> {
    let loader_result = load_one_by_model_id!(form_proposal_event_categories, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, EventCategoryType))
  }

  #[graphql(name = "user_con_profile_conventions")]
  async fn user_con_profile_conventions(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<ConventionType>, Error> {
    let loader_result = load_one_by_model_id!(form_user_con_profile_conventions, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, ConventionType))
  }
}

#[derive(MergedObject)]
#[graphql(name = "FormType")]
pub struct FormType(FormGlueFields, FormFormsFields);

impl ModelBackedType for FormType {
  type Model = forms::Model;

  fn new(model: Self::Model) -> Self {
    Self(
      FormGlueFields::new(model.clone()),
      FormFormsFields::new(model),
    )
  }

  fn get_model(&self) -> &Self::Model {
    self.0.get_model()
  }

  fn into_model(self) -> Self::Model {
    self.0.into_model()
  }
}
