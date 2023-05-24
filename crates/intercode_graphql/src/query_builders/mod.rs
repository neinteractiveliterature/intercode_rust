mod coupons_query_builder;
mod email_routes_query_builder;
mod event_proposals_query_builder;
mod events_query_builder;
mod orders_query_builder;
mod signup_requests_query_builder;
mod user_con_profiles_query_builder;

pub use coupons_query_builder::*;
pub use email_routes_query_builder::*;
pub use event_proposals_query_builder::*;
pub use events_query_builder::*;
use intercode_policies::{AuthorizationInfo, EntityPolicy, ReadManageAction};
pub use orders_query_builder::*;
pub use signup_requests_query_builder::*;
pub use user_con_profiles_query_builder::*;

use async_graphql::{Context, Error};
use sea_orm::{EntityTrait, Select};

use crate::api::interfaces::PaginationImplementation;

pub trait QueryBuilder {
  type Entity: EntityTrait;
  type Pagination: PaginationImplementation<Self::Entity>;

  fn apply_filters(&self, scope: Select<Self::Entity>) -> Select<Self::Entity>;
  fn apply_sorts(&self, scope: Select<Self::Entity>) -> Select<Self::Entity>;

  fn paginate(
    &self,
    scope: Select<Self::Entity>,
    page: Option<u64>,
    per_page: Option<u64>,
  ) -> Result<Self::Pagination, Error> {
    let scope = self.apply_filters(scope);
    let scope = self.apply_sorts(scope);

    Ok(Self::Pagination::new(Some(scope), page, per_page))
  }

  fn paginate_authorized<
    P: EntityPolicy<AuthorizationInfo, <Self::Entity as EntityTrait>::Model, Action = A>,
    A: From<ReadManageAction>,
  >(
    &self,
    ctx: &Context<'_>,
    scope: Select<Self::Entity>,
    page: Option<u64>,
    per_page: Option<u64>,
    _policy: P,
  ) -> Result<Self::Pagination, Error>
  where
    <Self::Entity as EntityTrait>::Model: Sync,
  {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;
    let scope = P::filter_scope(scope, authorization_info, &A::from(ReadManageAction::Read));
    self.paginate(scope, page, per_page)
  }
}
