use std::sync::Arc;

use async_graphql::{Context, Error, Object};
use intercode_entities::{cms_parent::CmsParent, cms_partials, conventions, root_sites};
use intercode_graphql_core::{
  liquid_renderer::LiquidRenderer, query_data::QueryData, ModelBackedType,
};
use liquid::object;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

#[derive(Default)]
pub struct QueryRootCmsFields;

impl QueryRootCmsFields {
  pub async fn cms_parent_by_domain<T: From<CmsParent>>(
    ctx: &Context<'_>,
    domain: &str,
  ) -> Result<T, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let convention = conventions::Entity::find()
      .filter(conventions::Column::Domain.eq(domain))
      .one(query_data.db())
      .await?;

    if let Some(convention) = convention {
      Ok(CmsParent::Convention(Box::new(convention)).into())
    } else {
      let root_site = <root_sites::Entity as EntityTrait>::find()
        .one(query_data.db())
        .await?;

      if let Some(root_site) = root_site {
        Ok(CmsParent::RootSite(Box::new(root_site)).into())
      } else {
        Err(Error::new("No root site found in database"))
      }
    }
  }

  pub async fn cms_parent_by_request_host<T: From<CmsParent>>(
    ctx: &Context<'_>,
  ) -> Result<T, Error> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(query_data.cms_parent().clone().into())
  }

  pub async fn root_site<T: ModelBackedType<Model = root_sites::Model>>(
    ctx: &Context<'_>,
  ) -> Result<T, Error> {
    let query_data = ctx.data::<QueryData>()?;

    let root_site = <root_sites::Entity as EntityTrait>::find()
      .one(query_data.db())
      .await?;

    if let Some(root_site) = root_site {
      Ok(T::new(root_site))
    } else {
      Err(Error::new("No root site found in database"))
    }
  }
}

#[Object]
impl QueryRootCmsFields {
  /// If there is a CMS partial on the root site called `account_form_text`, renders it to HTML
  /// and returns the result. Otherwise, returns null.
  ///
  /// This is used by the "update your account" pages as a way to clarify that your account is
  /// shared between multiple conventions.
  async fn account_form_content_html(&self, ctx: &Context<'_>) -> Result<Option<String>, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;

    let Some(account_form_text_template) = cms_partials::Entity::find()
      .filter(cms_partials::Column::ParentId.is_null())
      .filter(cms_partials::Column::ParentType.is_null())
      .filter(cms_partials::Column::Name.eq("account_form_text"))
      .one(query_data.db())
      .await?
      .and_then(|partial| partial.content)
    else {
      return Ok(None);
    };

    Ok(Some(
      liquid_renderer
        .render_liquid(account_form_text_template.as_str(), object!({}), None)
        .await?,
    ))
  }

  async fn preview_liquid(&self, ctx: &Context<'_>, content: String) -> Result<String, Error> {
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;
    liquid_renderer
      .render_liquid(content.as_str(), object!({}), None)
      .await
  }
}
