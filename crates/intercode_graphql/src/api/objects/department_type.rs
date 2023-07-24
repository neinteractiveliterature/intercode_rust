use async_graphql::*;
use intercode_entities::departments;
use intercode_graphql_core::{load_one_by_model_id, loader_result_to_many, model_backed_type};
use intercode_policies::{
  policies::DepartmentPolicy, ModelBackedTypeGuardablePolicy, ReadManageAction,
};

use crate::api::merged_objects::EventCategoryType;

model_backed_type!(DepartmentType, departments::Model);

#[Object(
  name = "Department",
  guard = "DepartmentPolicy::model_guard(ReadManageAction::Read, self)"
)]
impl DepartmentType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "event_categories")]
  async fn event_categories(&self, ctx: &Context<'_>) -> Result<Vec<EventCategoryType>> {
    let loader_result = load_one_by_model_id!(department_event_categories, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, EventCategoryType))
  }

  async fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }

  #[graphql(name = "proposal_description")]
  async fn proposal_description(&self) -> Option<&str> {
    self.model.proposal_description.as_deref()
  }
}
