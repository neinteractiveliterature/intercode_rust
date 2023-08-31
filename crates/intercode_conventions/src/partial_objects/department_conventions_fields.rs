use async_graphql::*;
use intercode_entities::{departments, event_categories};
use intercode_graphql_core::{load_one_by_model_id, model_backed_type};
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};
use seawater::loaders::ExpectModels;

use crate::policies::DepartmentPolicy;

model_backed_type!(DepartmentConventionsFields, departments::Model);

impl DepartmentConventionsFields {
  pub async fn event_categories(&self, ctx: &Context<'_>) -> Result<Vec<event_categories::Model>> {
    let loader_result = load_one_by_model_id!(department_event_categories, ctx, self)?;
    loader_result.expect_models().cloned()
  }
}

#[Object(guard = "DepartmentPolicy::model_guard(ReadManageAction::Read, self)")]
impl DepartmentConventionsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }

  #[graphql(name = "proposal_description")]
  async fn proposal_description(&self) -> Option<&str> {
    self.model.proposal_description.as_deref()
  }
}
