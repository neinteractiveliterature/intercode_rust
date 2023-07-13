use async_graphql::Union;

use super::{CmsLayoutType, CmsPartialType, PageType};

#[derive(Union)]
#[graphql(name = "CmsContent")]
pub enum CmsContentType {
  Page(PageType),
  Partial(CmsPartialType),
  Layout(CmsLayoutType),
}
