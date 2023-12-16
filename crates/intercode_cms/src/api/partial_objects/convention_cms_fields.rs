use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{cms_parent::CmsParentTrait, cms_partials, conventions};
use intercode_graphql_core::{
  liquid_renderer::LiquidRenderer, model_backed_type, query_data::QueryData,
};
use intercode_graphql_core::{load_one_by_model_id, loader_result_to_many};
use liquid::object;
use sea_orm::{ColumnTrait, QueryFilter};

use crate::api::objects::NotificationTemplateType;
use crate::{
  api::objects::{
    CmsContentType, CmsFileType, CmsGraphqlQueryType, CmsLayoutType, CmsNavigationItemType,
    CmsPartialType, CmsVariableType, PageType,
  },
  cms_parent_implementation::CmsParentImplementation,
  CmsRenderingContext,
};

use super::CmsContentGroupCmsFields;

model_backed_type!(ConventionCmsFields, conventions::Model);

impl ConventionCmsFields {
  pub async fn cms_content_groups(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsContentGroupCmsFields>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_content_groups(self, ctx).await
  }

  pub async fn cms_content_group(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<CmsContentGroupCmsFields, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_content_group(self, ctx, id).await
  }
}

#[Object]
impl ConventionCmsFields {
  pub async fn id(&self) -> ID {
    ID(self.model.id.to_string())
  }

  async fn cms_files(&self, ctx: &Context<'_>) -> Result<Vec<CmsFileType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_files(self, ctx).await
  }

  async fn cms_file(&self, ctx: &Context<'_>, id: ID) -> Result<CmsFileType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_file(self, ctx, id).await
  }

  async fn cms_graphql_queries(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsGraphqlQueryType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_graphql_queries(self, ctx).await
  }

  async fn cms_graphql_query(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<CmsGraphqlQueryType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_graphql_query(self, ctx, id).await
  }

  async fn cms_layouts(&self, ctx: &Context<'_>) -> Result<Vec<CmsLayoutType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_layouts(self, ctx).await
  }

  async fn cms_layout(&self, ctx: &Context<'_>, id: ID) -> Result<CmsLayoutType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_layout(self, ctx, id).await
  }

  async fn cms_navigation_items(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsNavigationItemType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_navigation_items(self, ctx).await
  }

  async fn cms_pages(&self, ctx: &Context<'_>) -> Result<Vec<PageType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_pages(self, ctx).await
  }

  async fn cms_page(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
    slug: Option<String>,
    root_page: Option<bool>,
  ) -> Result<PageType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_page(self, ctx, id, slug, root_page)
      .await
  }

  async fn cms_partials(&self, ctx: &Context<'_>) -> Result<Vec<CmsPartialType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_partials(self, ctx).await
  }

  async fn cms_variables(&self, ctx: &Context<'_>) -> Result<Vec<CmsVariableType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::cms_variables(self, ctx).await
  }

  async fn default_layout(&self, ctx: &Context<'_>) -> Result<CmsLayoutType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::default_layout(self, ctx).await
  }

  async fn effective_cms_layout(
    &self,
    ctx: &Context<'_>,
    path: String,
  ) -> Result<CmsLayoutType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::effective_cms_layout(self, ctx, path)
      .await
  }

  #[graphql(name = "notification_templates")]
  async fn notification_templates(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<NotificationTemplateType>> {
    let loader_result = load_one_by_model_id!(convention_notification_templates, ctx, self)?;
    Ok(loader_result_to_many!(
      loader_result,
      NotificationTemplateType
    ))
  }

  async fn root_page(&self, ctx: &Context<'_>) -> Result<PageType, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::root_page(self, ctx).await
  }

  async fn typeahead_search_cms_content(
    &self,
    ctx: &Context<'_>,
    name: Option<String>,
  ) -> Result<Vec<CmsContentType>, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::typeahead_search_cms_content(
      self, ctx, name,
    )
    .await
  }

  #[graphql(name = "pre_schedule_content_html")]
  async fn pre_schedule_content_html(&self, ctx: &Context<'_>) -> Result<Option<String>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;

    let partial = self
      .model
      .cms_partials()
      .filter(cms_partials::Column::Name.eq("pre_schedule_text"))
      .one(query_data.db())
      .await?;

    if let Some(partial) = partial {
      let cms_rendering_context =
        CmsRenderingContext::new(object!({}), query_data, liquid_renderer.as_ref());

      cms_rendering_context
        .render_liquid(&partial.content.unwrap_or_default(), None)
        .await
        .map(Some)
    } else {
      Ok(None)
    }
  }

  async fn preview_liquid(&self, ctx: &Context<'_>, content: String) -> Result<String, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::preview_liquid(self, ctx, content).await
  }

  async fn preview_markdown(
    &self,
    ctx: &Context<'_>,
    markdown: String,
    event_id: Option<ID>,
    event_proposal_id: Option<ID>,
  ) -> Result<String, Error> {
    <Self as CmsParentImplementation<conventions::Model>>::preview_markdown(
      self,
      ctx,
      markdown,
      event_id,
      event_proposal_id,
    )
    .await
  }
}

impl CmsParentImplementation<conventions::Model> for ConventionCmsFields {}
