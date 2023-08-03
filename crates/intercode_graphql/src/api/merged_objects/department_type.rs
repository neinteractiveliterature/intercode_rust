use async_graphql::*;
use intercode_conventions::{
  partial_objects::DepartmentConventionsFields, policies::DepartmentPolicy,
};
use intercode_entities::departments;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_policies::{ModelBackedTypeGuardablePolicy, ReadManageAction};

use crate::{api::merged_objects::EventCategoryType, merged_model_backed_type};

model_backed_type!(DepartmentGlueFields, departments::Model);

#[Object(guard = "DepartmentPolicy::model_guard(ReadManageAction::Read, self)")]
impl DepartmentGlueFields {
  #[graphql(name = "event_categories")]
  async fn event_categories(&self, ctx: &Context<'_>) -> Result<Vec<EventCategoryType>> {
    DepartmentConventionsFields::from_type(self.clone())
      .event_categories(ctx)
      .await
      .map(|res| res.into_iter().map(EventCategoryType::new).collect())
  }
}

merged_model_backed_type!(
  DepartmentType,
  departments::Model,
  "Department",
  DepartmentConventionsFields,
  DepartmentGlueFields
);
