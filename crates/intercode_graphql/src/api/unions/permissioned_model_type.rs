use async_graphql::Union;

use crate::api::objects::{CmsContentGroupType, ConventionType, EventCategoryType};

#[derive(Union)]
#[graphql(name = "PermissionedModel")]
pub enum PermissionedModelType {
  CmsContentGroup(CmsContentGroupType),
  Convention(ConventionType),
  EventCategory(EventCategoryType),
}
