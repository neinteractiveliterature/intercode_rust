use async_graphql::Union;

use crate::api::{
  merged_objects::EventCategoryType,
  objects::{CmsContentGroupType, ConventionType},
};

#[derive(Union)]
#[graphql(name = "PermissionedModel")]
pub enum PermissionedModelType {
  CmsContentGroup(CmsContentGroupType),
  Convention(ConventionType),
  EventCategory(EventCategoryType),
}
