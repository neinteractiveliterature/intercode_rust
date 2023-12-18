use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{cms_parent::CmsParentTrait, cms_partials, conventions};
use intercode_graphql_core::{
  liquid_renderer::LiquidRenderer, model_backed_type, query_data::QueryData,
};
use intercode_graphql_core::{load_one_by_model_id, loader_result_to_many};
use liquid::object;
use sea_orm::{ColumnTrait, QueryFilter};

use crate::api::objects::{LiquidAssignType, NotificationTemplateType};
use crate::{
  api::objects::{
    CmsContentType, CmsFileType, CmsGraphqlQueryType, CmsLayoutType, CmsNavigationItemType,
    CmsPartialType, CmsVariableType, PageType,
  },
  cms_parent_implementation::CmsParentImplementation,
  CmsRenderingContext,
};

model_backed_type!(ConventionCmsFields, conventions::Model);

impl CmsParentImplementation<conventions::Model> for ConventionCmsFields {}

#[Object]
impl ConventionCmsFields {
  async fn id(&self) -> ID {
    ID(self.model.id.to_string())
  }

  #[graphql(name = "clickwrap_agreement_html")]
  async fn clickwrap_agreement_html(&self, ctx: &Context<'_>) -> Result<Option<String>, Error> {
    let Some(clickwrap_agreement) = self.model.clickwrap_agreement.as_deref() else {
      return Ok(None);
    };

    let query_data = ctx.data::<QueryData>()?;
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;
    let cms_rendering_context =
      CmsRenderingContext::new(object!({}), query_data, liquid_renderer.as_ref());

    cms_rendering_context
      .render_liquid(clickwrap_agreement, None)
      .await
      .map(Some)
  }

  async fn cms_files(&self, ctx: &Context<'_>) -> Result<Vec<CmsFileType>, Error> {
    CmsParentImplementation::cms_files(self, ctx).await
  }

  async fn cms_file(&self, ctx: &Context<'_>, id: ID) -> Result<CmsFileType, Error> {
    CmsParentImplementation::cms_file(self, ctx, id).await
  }

  async fn cms_graphql_queries(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsGraphqlQueryType>, Error> {
    CmsParentImplementation::cms_graphql_queries(self, ctx).await
  }

  async fn cms_graphql_query(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<CmsGraphqlQueryType, Error> {
    CmsParentImplementation::cms_graphql_query(self, ctx, id).await
  }

  async fn cms_layouts(&self, ctx: &Context<'_>) -> Result<Vec<CmsLayoutType>, Error> {
    CmsParentImplementation::cms_layouts(self, ctx).await
  }

  async fn cms_layout(&self, ctx: &Context<'_>, id: ID) -> Result<CmsLayoutType, Error> {
    CmsParentImplementation::cms_layout(self, ctx, id).await
  }

  async fn cms_navigation_items(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<CmsNavigationItemType>, Error> {
    CmsParentImplementation::cms_navigation_items(self, ctx).await
  }

  async fn cms_pages(&self, ctx: &Context<'_>) -> Result<Vec<PageType>, Error> {
    CmsParentImplementation::cms_pages(self, ctx).await
  }

  async fn cms_page(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
    slug: Option<String>,
    root_page: Option<bool>,
  ) -> Result<PageType, Error> {
    CmsParentImplementation::cms_page(self, ctx, id, slug, root_page).await
  }

  async fn cms_partials(&self, ctx: &Context<'_>) -> Result<Vec<CmsPartialType>, Error> {
    CmsParentImplementation::cms_partials(self, ctx).await
  }

  async fn cms_variables(&self, ctx: &Context<'_>) -> Result<Vec<CmsVariableType>, Error> {
    CmsParentImplementation::cms_variables(self, ctx).await
  }

  async fn default_layout(&self, ctx: &Context<'_>) -> Result<CmsLayoutType, Error> {
    CmsParentImplementation::default_layout(self, ctx).await
  }

  async fn effective_cms_layout(
    &self,
    ctx: &Context<'_>,
    path: String,
  ) -> Result<CmsLayoutType, Error> {
    CmsParentImplementation::effective_cms_layout(self, ctx, path).await
  }

  async fn liquid_assigns(&self, ctx: &Context<'_>) -> Result<Vec<LiquidAssignType>> {
    CmsParentImplementation::liquid_assigns(self, ctx).await
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
    CmsParentImplementation::root_page(self, ctx).await
  }

  async fn typeahead_search_cms_content(
    &self,
    ctx: &Context<'_>,
    name: Option<String>,
  ) -> Result<Vec<CmsContentType>, Error> {
    CmsParentImplementation::typeahead_search_cms_content(self, ctx, name).await
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
    CmsParentImplementation::preview_liquid(self, ctx, content).await
  }

  async fn preview_markdown(
    &self,
    ctx: &Context<'_>,
    markdown: String,
    event_id: Option<ID>,
    event_proposal_id: Option<ID>,
  ) -> Result<String, Error> {
    CmsParentImplementation::preview_markdown(self, ctx, markdown, event_id, event_proposal_id)
      .await
  }
}
