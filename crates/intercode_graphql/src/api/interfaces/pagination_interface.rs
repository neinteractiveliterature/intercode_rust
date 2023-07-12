use async_graphql::{Context, Error, Interface};
use async_trait::async_trait;
use intercode_graphql_core::query_data::QueryData;
use intercode_policies::{AuthorizationInfo, EntityPolicy, ReadManageAction};
use intercode_query_builders::QueryBuilder;
use sea_orm::{ConnectionTrait, EntityTrait, Paginator, Select, SelectorTrait};

use crate::api::objects::{
  CouponsPaginationType, EmailRoutesPaginationType, EventProposalsPaginationType,
  EventsPaginationType, OrdersPaginationType, SignupRequestsPaginationType, SignupsPaginationType,
  UserConProfilesPaginationType,
};

#[derive(Interface)]
#[graphql(
  field(
    name = "total_entries",
    method = "total_entries",
    type = "u64",
    desc = "The total number of items in the paginated list (across all pages)"
  ),
  field(
    name = "total_pages",
    method = "total_pages",
    type = "u64",
    desc = "The total number of pages in the paginated list"
  ),
  field(
    name = "current_page",
    method = "current_page",
    type = "u64",
    desc = "The number of the page currently being returned in this query"
  ),
  field(
    name = "per_page",
    method = "per_page",
    type = "u64",
    desc = "The number of items per page currently being returned in this query"
  )
)]
/// PaginationInterface provides a way to use offset-based pagination on a list of objects. This
/// is useful for UIs such as Intercode's table views, which provide a way to jump between numbered
/// pages.
///
/// Page numbers in PaginationInterface are 1-based (so, the first page is page 1, then page 2,
/// etc.) The number of items per page can be controlled via the per_page argument on paginated
/// fields. It defaults to 20, and can go up to 200.
///
/// Offset-based pagination is different from
/// [Relay's cursor-based pagination](https://relay.dev/graphql/connections.htm) that is more
/// commonly used in GraphQL APIs. We chose to go with an offset-based approach due to our UI
/// needs, but if a cursor-based approach is desirable in the future, we may also implement Relay
/// connections alongside our existing pagination fields.
pub enum PaginationInterface {
  Coupons(CouponsPaginationType),
  EmailRoutes(EmailRoutesPaginationType),
  EventProposals(EventProposalsPaginationType),
  Events(EventsPaginationType),
  Orders(OrdersPaginationType),
  SignupRequests(SignupRequestsPaginationType),
  Signups(SignupsPaginationType),
  UserConProfiles(UserConProfilesPaginationType),
}

#[async_trait]
pub trait PaginationImplementation<Entity: EntityTrait + Send + Sync> {
  type Selector: SelectorTrait<Item = Entity::Model> + Send + Sync;

  fn new(scope: Option<Select<Entity>>, page: Option<u64>, per_page: Option<u64>) -> Self;

  fn paginator_and_page_size<'s, C: ConnectionTrait>(
    &'s self,
    db: &'s C,
  ) -> (Paginator<'s, C, Self::Selector>, u64);

  fn from_query_builder<B: QueryBuilder<Entity = Entity>>(
    query_builder: &B,
    scope: Select<Entity>,
    page: Option<u64>,
    per_page: Option<u64>,
  ) -> Self
  where
    Self: Sized,
  {
    let scope = query_builder.apply_filters(scope);
    let scope = query_builder.apply_sorts(scope);

    Self::new(Some(scope), page, per_page)
  }

  fn authorized_from_query_builder<
    B: QueryBuilder<Entity = Entity>,
    P: EntityPolicy<AuthorizationInfo, <Entity as EntityTrait>::Model, Action = A>,
    A: From<ReadManageAction>,
  >(
    query_builder: &B,
    ctx: &Context<'_>,
    scope: Select<Entity>,
    page: Option<u64>,
    per_page: Option<u64>,
    _policy: P,
  ) -> Result<Self, Error>
  where
    <Entity as EntityTrait>::Model: Sync,
    Self: Sized,
  {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let scope = P::filter_scope(scope, authorization_info, &A::from(ReadManageAction::Read));
    Ok(Self::from_query_builder(
      query_builder,
      scope,
      page,
      per_page,
    ))
  }

  async fn total_entries(&self, ctx: &Context) -> Result<u64, Error> {
    let db = ctx.data::<QueryData>()?.db();
    Ok(self.paginator_and_page_size(db).0.num_items().await?)
  }

  async fn total_pages(&self, ctx: &Context) -> Result<u64, Error> {
    let db = ctx.data::<QueryData>()?.db();
    Ok(self.paginator_and_page_size(db).0.num_pages().await?)
  }

  async fn current_page(&self, ctx: &Context) -> Result<u64, Error> {
    let db = ctx.data::<QueryData>()?.db();
    Ok(self.paginator_and_page_size(db).0.cur_page())
  }

  async fn per_page(&self, ctx: &Context) -> Result<u64, Error> {
    let db = ctx.data::<QueryData>()?.db();
    Ok(self.paginator_and_page_size(db).1)
  }
}
