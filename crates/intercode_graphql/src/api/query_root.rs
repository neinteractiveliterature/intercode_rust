use super::interfaces::CmsParentInterface;
use super::objects::{ConventionType, EventType, RootSiteType, UserType};
use crate::api::objects::ModelBackedType;
use crate::entity_relay_connection::RelayConnectable;
use crate::{QueryData, SchemaData};
use async_graphql::connection::{query, Connection};
use async_graphql::*;
use intercode_entities::cms_parent::CmsParent;
use intercode_entities::{events, oauth_applications, root_sites};
use liquid::object;
use sea_orm::{EntityTrait, PaginatorTrait};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
  pub async fn cms_parent_by_request_host(
    &self,
    ctx: &Context<'_>,
  ) -> Result<CmsParentInterface, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(match query_data.cms_parent.as_ref() {
      CmsParent::Convention(convention) => {
        CmsParentInterface::Convention(ConventionType::new(*convention.to_owned()))
      }
      CmsParent::RootSite(root_site) => {
        CmsParentInterface::RootSite(RootSiteType::new(*root_site.to_owned()))
      }
    })
  }

  async fn convention_by_request_host(&self, ctx: &Context<'_>) -> Result<ConventionType, Error> {
    let convention = self.convention_by_request_host_if_present(ctx).await?;

    match convention {
      Some(convention) => Ok(convention),
      None => Err(Error::new("No convention found for this domain name")),
    }
  }

  async fn convention_by_request_host_if_present(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<ConventionType>, Error> {
    let query_data = ctx.data::<QueryData>()?;

    match query_data.convention.as_ref() {
      Some(convention) => Ok(Some(ConventionType::new(convention.to_owned()))),
      None => Ok(None),
    }
  }

  async fn current_user(&self, ctx: &Context<'_>) -> Result<Option<UserType>, Error> {
    let query_data = ctx.data::<QueryData>()?;

    match query_data.current_user.as_ref() {
      Some(user) => Ok(Some(UserType::new(user.to_owned()))),
      None => Ok(None),
    }
  }

  async fn events(
    &self,
    ctx: &Context<'_>,
    after: Option<String>,
    before: Option<String>,
    first: Option<i32>,
    last: Option<i32>,
  ) -> Result<Connection<usize, EventType>> {
    query(
      after,
      before,
      first,
      last,
      |after, before, first, last| async move {
        let db = ctx.data::<SchemaData>()?.db.as_ref();

        let connection = events::Entity::find()
          .relay_connection(db, EventType::new, after, before, first, last)
          .to_connection()
          .await?;

        Ok::<_, Error>(connection)
      },
    )
    .await
  }

  async fn has_oauth_applications(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    let schema_data = ctx.data::<SchemaData>()?;

    let count = oauth_applications::Entity::find()
      .count(schema_data.db.as_ref())
      .await?;
    Ok(count > 0)
  }

  async fn preview_liquid(&self, ctx: &Context<'_>, content: String) -> Result<String, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    let query_data = ctx.data::<QueryData>()?;
    query_data
      .render_liquid(schema_data, content.as_str(), object!({}), None)
      .await
  }

  async fn root_site(&self, ctx: &Context<'_>) -> Result<RootSiteType, Error> {
    let schema_data = ctx.data::<SchemaData>()?;

    let root_site = root_sites::Entity::find()
      .one(schema_data.db.as_ref())
      .await?;

    if let Some(root_site) = root_site {
      Ok(RootSiteType::new(root_site))
    } else {
      Err(Error::new("No root site found in database"))
    }
  }
}
