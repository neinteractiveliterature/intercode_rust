use std::{env, sync::Arc};

use async_graphql::*;
use intercode_entities::cms_layouts;
use intercode_graphql_core::{
  liquid_renderer::LiquidRenderer, model_backed_type, query_data::QueryData,
};
use intercode_liquid::{cms_parent_partial_source::PreloadPartialsStrategy, react_component_tag};
use intercode_policies::{
  AuthorizationInfo, ModelBackedTypeGuardablePolicy, Policy, ReadManageAction,
};
use liquid::object;
use serde_json::json;

use crate::{api::policies::CmsLayoutPolicy, CmsRenderingContext};

model_backed_type!(CmsLayoutType, cms_layouts::Model);

const DEFAULT_NAVBAR_CLASSES: &str =
  "navbar-fixed-top navbar-expand-md mb-4 navbar-dark bg-intercode-blue";

#[Object(name = "CmsLayout")]
impl CmsLayoutType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(
    name = "admin_notes",
    guard = "CmsLayoutPolicy::model_guard(ReadManageAction::Manage, self)"
  )]
  async fn admin_notes(&self) -> Option<&str> {
    self.model.admin_notes.as_deref()
  }

  async fn content(&self) -> Option<&str> {
    self.model.content.as_deref()
  }

  #[graphql(name = "content_html")]
  #[allow(unused_variables)]
  async fn content_html(
    &self,
    ctx: &Context<'_>,
    path: Option<String>,
  ) -> Result<Option<String>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;

    let cms_rendering_context = CmsRenderingContext::new(
      object!({
        "content_for_head": "",
        "content_for_navbar": react_component_tag("NavigationBar", json!({
          "navbarClasses": self.model.navbar_classes.as_deref().unwrap_or(DEFAULT_NAVBAR_CLASSES)
        })),
        "content_for_layout": react_component_tag("AppRouter", json!({}))
      }),
      query_data,
      liquid_renderer.as_ref(),
    );

    cms_rendering_context
      .render_liquid(
        self.model.content.as_deref().unwrap_or(""),
        Some(PreloadPartialsStrategy::ByLayout(&self.model)),
      )
      .await
      .map(Some)
  }

  #[graphql(name = "content_html_with_placeholders")]
  #[allow(unused_variables)]
  async fn content_html_with_placeholders(
    &self,
    ctx: &Context<'_>,
    path: Option<String>,
  ) -> Result<Option<String>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;

    let styles_url = url::Url::parse(
      format!(
        "https://{}/packs/application-styles.js",
        env::var("ASSETS_HOST")?
      )
      .as_str(),
    )?;
    let cms_rendering_context = CmsRenderingContext::new(
      object!({
        "content_for_head": format!("<script src=\"{}\">{{{{ content_for_head }}}}", styles_url),
        "content_for_navbar": "{{ content_for_navbar }}",
        "content_for_layout": "{{ content_for_layout }}"
      }),
      query_data,
      liquid_renderer.as_ref(),
    );

    cms_rendering_context
      .render_liquid(
        self.model.content.as_deref().unwrap_or(""),
        Some(PreloadPartialsStrategy::ByLayout(&self.model)),
      )
      .await
      .map(Some)
  }

  #[graphql(name = "current_ability_can_delete")]
  async fn current_ability_can_delete(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;

    Ok(
      CmsLayoutPolicy::action_permitted(authorization_info, &ReadManageAction::Manage, &self.model)
        .await?,
    )
  }

  #[graphql(name = "current_ability_can_update")]
  async fn current_ability_can_update(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;

    Ok(
      CmsLayoutPolicy::action_permitted(authorization_info, &ReadManageAction::Manage, &self.model)
        .await?,
    )
  }

  async fn name(&self) -> Option<&str> {
    self.model.name.as_deref()
  }

  #[graphql(name = "navbar_classes")]
  async fn navbar_classes(&self) -> Option<&str> {
    self.model.navbar_classes.as_deref()
  }
}
