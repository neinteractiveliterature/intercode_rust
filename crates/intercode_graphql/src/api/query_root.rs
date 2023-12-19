use super::interfaces::CmsParentInterface;
use super::merged_objects::{
  AbilityType, ConventionType, EventType, OrganizationType, RootSiteType, UserConProfileType,
  UserType,
};
use async_graphql::connection::Connection;
use async_graphql::*;
use intercode_cms::api::partial_objects::QueryRootCmsFields;
use intercode_conventions::partial_objects::QueryRootConventionsFields;
use intercode_conventions::query_builders::ConventionFiltersInput;
use intercode_email::partial_objects::QueryRootEmailFields;
use intercode_events::partial_objects::QueryRootEventsFields;
use intercode_graphql_core::entity_relay_connection::type_converting_query;
use intercode_graphql_core::{ModelBackedType, ModelPaginator};
use intercode_query_builders::sort_input::SortInput;
use intercode_users::partial_objects::QueryRootUsersFields;
use intercode_users::query_builders::UserFiltersInput;

#[derive(Default)]
pub struct QueryRootGlueFields;

#[Object]
impl QueryRootGlueFields {
  pub async fn assumed_identity_from_profile(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<UserConProfileType>> {
    QueryRootUsersFields::assumed_identity_from_profile(ctx)
      .await
      .map(|res| res.map(UserConProfileType::from_type))
  }

  pub async fn cms_parent_by_domain(
    &self,
    ctx: &Context<'_>,
    domain: String,
  ) -> Result<CmsParentInterface, Error> {
    QueryRootCmsFields::cms_parent_by_domain(ctx, &domain).await
  }

  pub async fn cms_parent_by_request_host(
    &self,
    ctx: &Context<'_>,
  ) -> Result<CmsParentInterface, Error> {
    QueryRootCmsFields::cms_parent_by_request_host(ctx).await
  }

  async fn convention_by_domain(
    &self,
    ctx: &Context<'_>,
    domain: String,
  ) -> Result<ConventionType, Error> {
    QueryRootConventionsFields::convention_by_domain(ctx, domain).await
  }

  async fn convention_by_id(&self, ctx: &Context<'_>, id: ID) -> Result<ConventionType, Error> {
    QueryRootConventionsFields::convention_by_id(ctx, id).await
  }

  async fn convention_by_request_host(&self, ctx: &Context<'_>) -> Result<ConventionType, Error> {
    QueryRootConventionsFields::convention_by_request_host(ctx).await
  }

  async fn convention_by_request_host_if_present(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<ConventionType>, Error> {
    QueryRootConventionsFields::convention_by_request_host_if_present(ctx).await
  }

  #[graphql(name = "conventions_paginated")]
  pub async fn conventions_paginated(
    &self,
    filters: Option<ConventionFiltersInput>,
    sort: Option<Vec<SortInput>>,
    page: Option<u64>,
    per_page: Option<u64>,
  ) -> ModelPaginator<ConventionType> {
    QueryRootConventionsFields::conventions_paginated(filters, sort, page, per_page).await
  }

  pub async fn current_ability(&self, ctx: &Context<'_>) -> Result<AbilityType> {
    QueryRootUsersFields::current_ability(ctx)
      .await
      .map(AbilityType::from)
  }

  pub async fn current_user(&self, ctx: &Context<'_>) -> Result<Option<UserType>, Error> {
    QueryRootUsersFields::current_user(ctx)
      .await
      .map(|res| res.map(UserType::from_type))
  }

  async fn events(
    &self,
    ctx: &Context<'_>,
    after: Option<String>,
    before: Option<String>,
    first: Option<i32>,
    last: Option<i32>,
  ) -> Result<Connection<u64, EventType>> {
    type_converting_query(after, before, first, last, |after, before, first, last| {
      QueryRootEventsFields::events(ctx, after, before, first, last)
    })
    .await
  }

  async fn organizations(&self, ctx: &Context<'_>) -> Result<Vec<OrganizationType>> {
    QueryRootConventionsFields::organizations(ctx).await
  }

  async fn root_site(&self, ctx: &Context<'_>) -> Result<RootSiteType, Error> {
    QueryRootCmsFields::root_site(ctx).await
  }

  pub async fn user(&self, ctx: &Context<'_>, id: Option<ID>) -> Result<UserType, Error> {
    UserType::from_future_result(QueryRootUsersFields::user(ctx, id)).await
  }

  pub async fn users(
    &self,
    ctx: &Context<'_>,
    ids: Option<Vec<ID>>,
  ) -> Result<Vec<UserType>, Error> {
    UserType::from_many_future_result(QueryRootUsersFields::users(ctx, ids)).await
  }

  #[graphql(name = "users_paginated")]
  pub async fn users_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<UserFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<UserType>, Error> {
    Ok(
      QueryRootUsersFields::users_paginated(ctx, page, per_page, filters, sort)
        .await?
        .into_type(),
    )
  }
}

#[derive(MergedObject, Default)]
#[graphql(name = "Query")]
pub struct QueryRoot(
  QueryRootGlueFields,
  QueryRootCmsFields,
  QueryRootEmailFields,
  QueryRootUsersFields,
);
