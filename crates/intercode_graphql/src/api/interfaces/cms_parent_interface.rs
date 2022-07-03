use async_graphql::{async_trait::async_trait, Context, Error, Interface, ID};
use intercode_entities::{cms_parent::CmsParentTrait, pages};
use sea_orm::{ColumnTrait, QueryFilter};

use crate::{
  api::objects::{
    CmsLayoutType, CmsNavigationItemType, ConventionType, ModelBackedType, PageType, RootSiteType,
  },
  SchemaData,
};

#[derive(Interface)]
#[graphql(
  field(name = "id", type = "ID"),
  field(name = "cms_navigation_items", type = "Vec<CmsNavigationItemType>"),
  field(name = "default_layout", type = "CmsLayoutType"),
  field(
    name = "effective_cms_layout",
    type = "CmsLayoutType",
    arg(name = "path", type = "String")
  ),
  field(
    name = "cms_page",
    type = "PageType",
    arg(name = "id", type = "Option<ID>"),
    arg(name = "slug", type = "Option<String>"),
    arg(name = "root_page", type = "Option<bool>")
  )
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

#[async_trait]
pub trait CmsParentImplementation<M>
where
  Self: ModelBackedType<M>,
  M: CmsParentTrait + sea_orm::ModelTrait + Sync,
{
  async fn cms_page(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
    slug: Option<String>,
    root_page: Option<bool>,
  ) -> Result<PageType, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    let pages = if let Some(id) = id {
      self
        .get_model()
        .pages()
        .filter(pages::Column::Id.eq(id.parse::<i64>()?))
    } else if let Some(slug) = slug {
      self
        .get_model()
        .pages()
        .filter(pages::Column::Slug.eq(slug))
    } else if Some(true) == root_page {
      self.get_model().root_page()
    } else {
      return Err(Error::new(
        "cmsPage requires either an id, slug, or root_page parameter",
      ));
    };

    pages
      .one(schema_data.db.as_ref())
      .await?
      .ok_or_else(|| Error::new("Page not found"))
      .map(PageType::new)
  }

  async fn cms_navigation_items(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsNavigationItemType>, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    Ok(
      self
        .get_model()
        .cms_navigation_items()
        .all(schema_data.db.as_ref())
        .await?
        .iter()
        .map(|item| CmsNavigationItemType::new(item.to_owned()))
        .collect(),
    )
  }

  async fn default_layout(&self, ctx: &Context<'_>) -> Result<CmsLayoutType, Error> {
    let schema_data = ctx.data::<SchemaData>()?;

    self
      .get_model()
      .default_layout()
      .one(schema_data.db.as_ref())
      .await?
      .ok_or_else(|| Error::new("Default layout not found for root site"))
      .map(CmsLayoutType::new)
  }

  async fn effective_cms_layout(
    &self,
    ctx: &Context<'_>,
    path: String,
  ) -> Result<CmsLayoutType, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    self
      .get_model()
      .effective_cms_layout(path.as_str(), schema_data.db.as_ref())
      .await
      .map(|layout| CmsLayoutType::new(layout))
      .map_err(|db_err| Error::new(db_err.to_string()))
  }
}
