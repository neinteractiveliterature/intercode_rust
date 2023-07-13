use std::sync::Arc;

use async_graphql::{Context, Error, ID};
use async_trait::async_trait;
use futures::{try_join, TryFutureExt};
use intercode_entities::{
  cms_content_groups, cms_files, cms_graphql_queries, cms_layouts, cms_parent::CmsParentTrait,
  pages,
};
use intercode_graphql_core::{
  liquid_renderer::LiquidRenderer, query_data::QueryData, ModelBackedType,
};
use intercode_liquid::render_markdown;
use sea_orm::{ColumnTrait, QueryFilter};

use crate::{
  api::objects::{
    CmsContentType, CmsFileType, CmsGraphqlQueryType, CmsLayoutType, CmsNavigationItemType,
    CmsPartialType, CmsVariableType, LiquidAssignType, PageType,
  },
  api::partial_objects::CmsContentGroupCmsFields,
  CmsRenderingContext,
};

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
            .all(query_data.db())
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
            .one(query_data.db())
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
  assoc_getter!(cms_content_groups, CmsContentGroupCmsFields);
  id_getter!(
    cms_content_group,
    cms_content_groups,
    CmsContentGroupCmsFields
  );

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
        .all(query_data.db())
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
      .one(query_data.db())
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
      .one(query_data.db())
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
      .effective_cms_layout(path.as_str(), query_data.db())
      .await
      .map(CmsLayoutType::new)
      .map_err(|db_err| Error::new(db_err.to_string()))
  }

  async fn liquid_assigns(&self, ctx: &Context<'_>) -> Result<Vec<LiquidAssignType>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;
    let cms_rendering_context =
      CmsRenderingContext::new(liquid::object!({}), query_data, liquid_renderer.as_ref());

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
      CmsRenderingContext::new(liquid::object!({}), query_data, liquid_renderer.as_ref());

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
    Ok(render_markdown(&markdown, &Default::default()))
  }

  async fn root_page(&self, ctx: &Context<'_>) -> Result<PageType, Error> {
    let query_data = ctx.data::<QueryData>()?;
    self
      .get_model()
      .root_page()
      .one(query_data.db())
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
