use std::sync::Arc;

use async_graphql::{Context, Error, Object};
use intercode_entities::{cms_parent::CmsParent, root_sites};
use intercode_graphql_core::{
  liquid_renderer::LiquidRenderer, query_data::QueryData, ModelBackedType,
};
use liquid::object;
use sea_orm::EntityTrait;

use super::RootSiteCmsFields;

#[derive(Default)]
pub struct QueryRootCmsFields;

impl QueryRootCmsFields {
  pub async fn cms_parent_by_request_host(ctx: &Context<'_>) -> Result<CmsParent, Error> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(query_data.cms_parent().clone())
  }

  pub async fn root_site(ctx: &Context<'_>) -> Result<RootSiteCmsFields, Error> {
    let query_data = ctx.data::<QueryData>()?;

    let root_site = root_sites::Entity::find().one(query_data.db()).await?;

    if let Some(root_site) = root_site {
      Ok(RootSiteCmsFields::new(root_site))
    } else {
      Err(Error::new("No root site found in database"))
    }
  }
}

#[Object]
impl QueryRootCmsFields {
  async fn preview_liquid(&self, ctx: &Context<'_>, content: String) -> Result<String, Error> {
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;
    liquid_renderer
      .render_liquid(content.as_str(), object!({}), None)
      .await
  }
}
