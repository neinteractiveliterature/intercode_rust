use async_graphql::Union;
use intercode_graphql_core::ModelBackedType;
use intercode_graphql_loaders::permissioned_models_loader::PermissionedModel;

use crate::api::merged_objects::{CmsContentGroupType, ConventionType, EventCategoryType};

#[derive(Union)]
#[graphql(name = "PermissionedModel")]
pub enum PermissionedModelType {
  CmsContentGroup(CmsContentGroupType),
  Convention(ConventionType),
  EventCategory(EventCategoryType),
}

impl From<PermissionedModel> for PermissionedModelType {
  fn from(value: PermissionedModel) -> Self {
    match value {
      PermissionedModel::CmsContentGroup(model) => {
        PermissionedModelType::CmsContentGroup(CmsContentGroupType::new(model))
      }
      PermissionedModel::Convention(model) => {
        PermissionedModelType::Convention(ConventionType::new(model))
      }
      PermissionedModel::EventCategory(model) => {
        PermissionedModelType::EventCategory(EventCategoryType::new(model))
      }
    }
  }
}
