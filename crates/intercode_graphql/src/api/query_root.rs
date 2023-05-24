use std::borrow::Cow;
use std::sync::Arc;

use super::inputs::{EmailRouteFiltersInput, SortInput};
use super::interfaces::CmsParentInterface;
use super::objects::{
  AbilityType, ConventionType, EmailRoutesPaginationType, EventType, OrganizationType,
  RootSiteType, UserConProfileType, UserType,
};
use crate::api::objects::ModelBackedType;
use crate::entity_relay_connection::RelayConnectable;
use crate::query_builders::{EmailRoutesQueryBuilder, QueryBuilder};
use crate::{LiquidRenderer, QueryData};
use async_graphql::connection::{query, Connection};
use async_graphql::*;
use intercode_entities::cms_parent::CmsParent;
use intercode_entities::{email_routes, events, oauth_applications, organizations, root_sites};
use intercode_policies::policies::EmailRoutePolicy;
use intercode_policies::{AuthorizationInfo, EntityPolicy, ReadManageAction};
use itertools::Itertools;
use liquid::object;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};

pub struct QueryRoot;

#[Object(name = "Query")]
impl QueryRoot {
  pub async fn assumed_identity_from_profile(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<UserConProfileType>> {
    Ok(
      ctx
        .data::<AuthorizationInfo>()?
        .assumed_identity_from_profile
        .as_ref()
        .map(|profile| UserConProfileType::new(profile.clone())),
    )
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

  #[graphql(name = "email_routes_paginated")]
  async fn email_routes_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<EmailRouteFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<EmailRoutesPaginationType, Error> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let scope = email_routes::Entity::find().filter(
      email_routes::Column::Id.in_subquery(
        sea_orm::QuerySelect::query(
          &mut EmailRoutePolicy::accessible_to(authorization_info, &ReadManageAction::Read)
            .select_only()
            .column(email_routes::Column::Id),
        )
        .take(),
      ),
    );
    EmailRoutesQueryBuilder::new(filters, sort).paginate(scope, page, per_page)
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
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;
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
