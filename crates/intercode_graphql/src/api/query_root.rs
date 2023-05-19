use std::borrow::Cow;

use super::interfaces::CmsParentInterface;
use super::objects::{
  AbilityType, ConventionType, EventType, OrganizationType, RootSiteType, UserConProfileType,
  UserType,
};
use crate::api::objects::ModelBackedType;
use crate::entity_relay_connection::RelayConnectable;
use crate::{LiquidRenderer, QueryData};
use async_graphql::connection::{query, Connection};
use async_graphql::*;
use intercode_entities::cms_parent::CmsParent;
use intercode_entities::{events, oauth_applications, organizations, root_sites};
use intercode_policies::AuthorizationInfo;
use itertools::Itertools;
use liquid::object;
use sea_orm::{EntityTrait, PaginatorTrait};

pub struct QueryRoot;

#[Object(name = "Query")]
impl QueryRoot {
  pub async fn assumed_identity_from_profile(
    &self,
    _ctx: &Context<'_>,
  ) -> Option<UserConProfileType> {
    // TODO
    None
  }

  pub async fn cms_parent_by_request_host(
    &self,
    ctx: &Context<'_>,
  ) -> Result<CmsParentInterface, Error> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(match query_data.cms_parent() {
      CmsParent::Convention(convention) => {
        CmsParentInterface::Convention(Box::new(ConventionType::new(*convention.to_owned())))
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

    match query_data.convention() {
      Some(convention) => Ok(Some(ConventionType::new(convention.to_owned()))),
      None => Ok(None),
    }
  }

  async fn current_ability<'a>(&'a self, ctx: &'a Context<'a>) -> Result<AbilityType<'a>> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    Ok(AbilityType::new(Cow::Borrowed(authorization_info)))
  }

  async fn current_user(&self, ctx: &Context<'_>) -> Result<Option<UserType>, Error> {
    let query_data = ctx.data::<QueryData>()?;

    match query_data.current_user() {
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
  ) -> Result<Connection<u64, EventType>> {
    query(
      after,
      before,
      first,
      last,
      |after, before, first, last| async move {
        let db = ctx.data::<QueryData>()?.db();

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
    let query_data = ctx.data::<QueryData>()?;

    let count = oauth_applications::Entity::find()
      .count(query_data.db())
      .await?;
    Ok(count > 0)
  }

  async fn organizations(&self, ctx: &Context<'_>) -> Result<Vec<OrganizationType>> {
    let query_data = ctx.data::<QueryData>()?;

    Ok(
      organizations::Entity::find()
        .all(query_data.db())
        .await?
        .into_iter()
        .map(OrganizationType::new)
        .collect_vec(),
    )
  }

  async fn preview_liquid(&self, ctx: &Context<'_>, content: String) -> Result<String, Error> {
    let liquid_renderer = ctx.data::<Box<dyn LiquidRenderer>>()?;
    liquid_renderer
      .render_liquid(content.as_str(), object!({}), None)
      .await
  }

  async fn root_site(&self, ctx: &Context<'_>) -> Result<RootSiteType, Error> {
    let query_data = ctx.data::<QueryData>()?;

    let root_site = root_sites::Entity::find().one(query_data.db()).await?;

    if let Some(root_site) = root_site {
      Ok(RootSiteType::new(root_site))
    } else {
      Err(Error::new("No root site found in database"))
    }
  }
}
