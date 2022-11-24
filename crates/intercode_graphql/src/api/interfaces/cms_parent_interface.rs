use std::sync::Arc;

use async_graphql::{
  async_trait::async_trait,
  futures_util::{try_join, TryFutureExt},
  Context, Error, Interface, ID,
};
use intercode_entities::{
  cms_content_groups, cms_files, cms_graphql_queries, cms_layouts, cms_parent::CmsParentTrait,
  pages,
};
use intercode_liquid::render_markdown;
use sea_orm::{ColumnTrait, QueryFilter};

use crate::{
  api::objects::{
    CmsContentGroupType, CmsContentType, CmsFileType, CmsGraphqlQueryType, CmsLayoutType,
    CmsNavigationItemType, CmsPartialType, CmsVariableType, ConventionType, LiquidAssignType,
    ModelBackedType, PageType, RootSiteType, SearchResultType,
  },
  cms_rendering_context::CmsRenderingContext,
  LiquidRenderer, QueryData,
};

#[derive(Interface)]
#[graphql(
  field(name = "id", type = "ID"),
  field(name = "cms_content_groups", type = "Vec<CmsContentGroupType>"),
  field(
    name = "cms_content_group",
    type = "CmsContentGroupType",
    arg(name = "id", type = "ID")
  ),
  field(name = "cms_files", type = "Vec<CmsFileType>"),
  field(name = "cms_graphql_queries", type = "Vec<CmsGraphqlQueryType>"),
  field(name = "cms_layouts", type = "Vec<CmsLayoutType>"),
  field(name = "cms_navigation_items", type = "Vec<CmsNavigationItemType>"),
  field(name = "cms_pages", type = "Vec<PageType>"),
  field(
    name = "cms_page",
    type = "PageType",
    arg(name = "id", type = "Option<ID>"),
    arg(name = "slug", type = "Option<String>"),
    arg(name = "root_page", type = "Option<bool>")
  ),
  field(name = "cms_partials", type = "Vec<CmsPartialType>"),
  field(name = "cms_variables", type = "Vec<CmsVariableType>"),
  field(name = "default_layout", type = "CmsLayoutType"),
  field(
    name = "effective_cms_layout",
    type = "CmsLayoutType",
    arg(name = "path", type = "String")
  ),
  field(
    name = "full_text_search",
    type = "SearchResultType",
    arg(name = "query", type = "String")
  ),
  field(name = "liquid_assigns", type = "Vec<LiquidAssignType>"),
  field(
    name = "preview_markdown",
    type = "String",
    arg(name = "markdown", type = "String"),
    arg(name = "event_id", type = "Option<ID>"),
    arg(name = "event_proposal_id", type = "Option<ID>")
  ),
  field(
    name = "preview_liquid",
    type = "String",
    arg(name = "content", type = "String"),
  ),
  field(name = "root_page", type = "PageType"),
  field(
    name = "typeahead_search_cms_content",
    type = "Vec<CmsContentType>",
    arg(name = "name", type = "Option<String>")
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
  Convention(Box<ConventionType>),
}

macro_rules! assoc_getter {
  ($name: ident, $ty: ident) => {
    fn $name<'life0, 'async_trait>(
      &'life0 self,
      ctx: &'async_trait Context<'_>,
    ) -> std::pin::Pin<
      Box<dyn std::future::Future<Output = Result<Vec<$ty>, Error>> + Send + 'async_trait>,
    >
    where
      'life0: 'async_trait,
      Self: Sync + 'async_trait,
    {
      Box::pin(async move {
        let query_data = ctx.data::<QueryData>()?;
        Ok(
          self
            .get_model()
            .$name()
            .all(query_data.db.as_ref())
            .await?
            .iter()
            .map(|item| $ty::new(item.to_owned()))
            .collect(),
        )
      })
    }
  };
}

macro_rules! id_getter {
  ($name: ident, $model_name: ident, $ty: ident) => {
    fn $name<'life0, 'async_trait>(
      &'life0 self,
      ctx: &'async_trait Context<'_>,
      id: ID,
    ) -> std::pin::Pin<
      Box<dyn std::future::Future<Output = Result<$ty, Error>> + Send + 'async_trait>,
    >
    where
      'life0: 'async_trait,
      Self: Sync + 'async_trait,
    {
      Box::pin(async move {
        let query_data = ctx.data::<QueryData>()?;
        let id = id.parse::<i64>()?;
        Ok($ty::new(
          self
            .get_model()
            .$model_name()
            .filter($model_name::Column::Id.eq(id))
            .one(query_data.db.as_ref())
            .await?
            .ok_or_else(|| Error::new(format!("{} {} not found", stringify!($name), id)))?,
        ))
      })
    }
  };
}

#[async_trait]
pub trait CmsParentImplementation<M>
where
  Self: ModelBackedType<Model = M>,
  M: CmsParentTrait + sea_orm::ModelTrait + Sync,
{
  assoc_getter!(cms_content_groups, CmsContentGroupType);
  id_getter!(cms_content_group, cms_content_groups, CmsContentGroupType);

  assoc_getter!(cms_files, CmsFileType);
  id_getter!(cms_file, cms_files, CmsFileType);

  assoc_getter!(cms_graphql_queries, CmsGraphqlQueryType);
  id_getter!(cms_graphql_query, cms_graphql_queries, CmsGraphqlQueryType);

  assoc_getter!(cms_layouts, CmsLayoutType);
  id_getter!(cms_layout, cms_layouts, CmsLayoutType);

  assoc_getter!(cms_navigation_items, CmsNavigationItemType);

  assoc_getter!(cms_partials, CmsPartialType);

  async fn cms_pages(&self, ctx: &Context<'_>) -> Result<Vec<PageType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(
      self
        .get_model()
        .pages()
        .all(query_data.db.as_ref())
        .await?
        .iter()
        .map(|item| PageType::new(item.to_owned()))
        .collect(),
    )
  }

  async fn cms_page(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
    slug: Option<String>,
    root_page: Option<bool>,
  ) -> Result<PageType, Error> {
    let query_data = ctx.data::<QueryData>()?;
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
      .one(query_data.db.as_ref())
      .await?
      .ok_or_else(|| Error::new("Page not found"))
      .map(PageType::new)
  }

  assoc_getter!(cms_variables, CmsVariableType);

  async fn default_layout(&self, ctx: &Context<'_>) -> Result<CmsLayoutType, Error> {
    let query_data = ctx.data::<QueryData>()?;

    self
      .get_model()
      .default_layout()
      .one(query_data.db.as_ref())
      .await?
      .ok_or_else(|| Error::new("Default layout not found for root site"))
      .map(CmsLayoutType::new)
  }

  async fn effective_cms_layout(
    &self,
    ctx: &Context<'_>,
    path: String,
  ) -> Result<CmsLayoutType, Error> {
    let query_data = ctx.data::<QueryData>()?;
    self
      .get_model()
      .effective_cms_layout(path.as_str(), query_data.db.as_ref())
      .await
      .map(CmsLayoutType::new)
      .map_err(|db_err| Error::new(db_err.to_string()))
  }

  async fn full_text_search(
    &self,
    _ctx: &Context<'_>,
    _query: String,
  ) -> Result<SearchResultType, Error> {
    // TODO
    Ok(SearchResultType)
  }

  async fn liquid_assigns(&self, ctx: &Context<'_>) -> Result<Vec<LiquidAssignType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;
    let cms_rendering_context =
      CmsRenderingContext::new(liquid::object!({}), query_data, liquid_renderer.clone());

    let (builtins, cms_variables) = try_join!(
      liquid_renderer.builtin_globals(),
      cms_rendering_context
        .cms_variables()
        .map_err(|err| Error::new(err.to_string()))
    )?;

    Ok(
      builtins
        .iter()
        .map(|(key, value)| LiquidAssignType::from_value_view(key.to_string(), value))
        .chain(
          cms_variables
            .iter()
            .map(LiquidAssignType::from_cms_variable),
        )
        .collect(),
    )
  }

  async fn preview_liquid(&self, ctx: &Context<'_>, content: String) -> Result<String, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;
    let cms_rendering_context =
      CmsRenderingContext::new(liquid::object!({}), query_data, liquid_renderer.clone());

    cms_rendering_context.render_liquid(&content, None).await
  }

  async fn preview_markdown(
    &self,
    _ctx: &Context<'_>,
    markdown: String,
    _event_id: Option<ID>,
    _event_proposal_id: Option<ID>,
  ) -> Result<String, Error> {
    // TODO find images for event or event proposal
    Ok(render_markdown(&markdown))
  }

  async fn root_page(&self, ctx: &Context<'_>) -> Result<PageType, Error> {
    let query_data = ctx.data::<QueryData>()?;
    self
      .get_model()
      .root_page()
      .one(query_data.db.as_ref())
      .await?
      .ok_or_else(|| Error::new("root page not found"))
      .map(PageType::new)
  }

  async fn typeahead_search_cms_content(
    &self,
    _ctx: &Context<'_>,
    _name: Option<String>,
  ) -> Result<Vec<CmsContentType>, Error> {
    // TODO
    Ok(vec![])
  }
}
