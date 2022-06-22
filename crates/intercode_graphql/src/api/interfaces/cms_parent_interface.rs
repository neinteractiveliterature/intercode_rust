use async_graphql::{Interface, ID};

use crate::api::objects::{CmsLayoutType, CmsNavigationItemType, ConventionType, RootSiteType};

#[derive(Interface)]
#[graphql(
  field(name = "id", type = "ID"),
  field(name = "cms_navigation_items", type = "Vec<CmsNavigationItemType>"),
  field(name = "default_layout", type = "CmsLayoutType")
)]
/// A CMS parent is a web site managed by Intercode. It acts as a container for CMS content, such
/// as pages, partials, files, layouts, variables, content groups, and user-defined GraphQL queries.
///
/// Most CMS parents are conventions, so their content will be convention-specific and scoped to
/// that convention's domain name. The exception to this is the root site, which is what Intercode
/// renders when there is no convention associated with the current domain name. (See the RootSite
/// object for more details about this.)
pub enum CmsParentInterface {
  RootSite(RootSiteType),
  Convention(ConventionType),
}
