use async_graphql::Union;

use crate::api::{
  merged_objects::{ConventionType, EventCategoryType},
  objects::CmsContentGroupType,
};

#[derive(Union)]
#[graphql(name = "PermissionedModel")]
pub enum PermissionedModelType {
  CmsContentGroup(CmsContentGroupType),
  Convention(ConventionType),
  EventCategory(EventCategoryType),
}
