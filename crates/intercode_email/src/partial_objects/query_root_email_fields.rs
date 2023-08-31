use async_graphql::*;
use intercode_entities::email_routes;
use intercode_graphql_core::ModelPaginator;
use intercode_policies::AuthorizedFromQueryBuilder;
use intercode_query_builders::sort_input::SortInput;
use sea_orm::EntityTrait;

use crate::{
  objects::EmailRouteType,
  policies::EmailRoutePolicy,
  query_builders::{EmailRouteFiltersInput, EmailRoutesQueryBuilder},
};

#[derive(Default)]
pub struct QueryRootEmailFields;

#[Object]
impl QueryRootEmailFields {
  #[graphql(name = "email_routes_paginated")]
  async fn email_routes_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    #[graphql(name = "per_page")] per_page: Option<u64>,
    filters: Option<EmailRouteFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<EmailRouteType>, Error> {
    ModelPaginator::authorized_from_query_builder(
      &EmailRoutesQueryBuilder::new(filters, sort),
      ctx,
      email_routes::Entity::find(),
      page,
      per_page,
      EmailRoutePolicy,
    )
  }
}
