use async_graphql::Union;

use super::{CmsLayoutType, CmsPartialType, PageType};

#[derive(Union)]
pub enum CmsContentType {
  Page(PageType),
  Partial(CmsPartialType),
  Layout(CmsLayoutType),
}
