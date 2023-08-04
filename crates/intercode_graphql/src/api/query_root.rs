use super::interfaces::CmsParentInterface;
use super::merged_objects::{
  AbilityType, ConventionType, EventType, OrganizationType, RootSiteType, UserConProfileType,
  UserType,
};
use async_graphql::connection::Connection;
use async_graphql::*;
use intercode_cms::api::partial_objects::QueryRootCmsFields;
use intercode_conventions::partial_objects::QueryRootConventionsFields;
use intercode_email::partial_objects::QueryRootEmailFields;
use intercode_events::partial_objects::QueryRootEventsFields;
use intercode_graphql_core::entity_relay_connection::type_converting_query;
use intercode_graphql_core::ModelBackedType;
use intercode_users::partial_objects::QueryRootUsersFields;

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

  pub async fn cms_parent_by_request_host(
    &self,
    ctx: &Context<'_>,
  ) -> Result<CmsParentInterface, Error> {
    QueryRootCmsFields::cms_parent_by_request_host(ctx)
      .await
      .map(CmsParentInterface::from)
  }

  async fn convention_by_request_host(&self, ctx: &Context<'_>) -> Result<ConventionType, Error> {
    QueryRootConventionsFields::convention_by_request_host(ctx)
      .await
      .map(ConventionType::from_type)
  }

  async fn convention_by_request_host_if_present(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<ConventionType>, Error> {
    QueryRootConventionsFields::convention_by_request_host_if_present(ctx)
      .await
      .map(|res| res.map(ConventionType::from_type))
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
    QueryRootConventionsFields::organizations(ctx)
      .await
      .map(|res| res.into_iter().map(OrganizationType::from_type).collect())
  }

  async fn root_site(&self, ctx: &Context<'_>) -> Result<RootSiteType, Error> {
    QueryRootCmsFields::root_site(ctx)
      .await
      .map(RootSiteType::from_type)
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
