mod coupons_query_builder;
mod event_proposals_query_builder;
mod orders_query_builder;
mod signup_requests_query_builder;

pub use coupons_query_builder::*;
pub use event_proposals_query_builder::*;
pub use orders_query_builder::*;
pub use signup_requests_query_builder::*;

use async_graphql::{Context, Error};
use sea_orm::{EntityTrait, Select};

use crate::api::interfaces::PaginationImplementation;

pub trait QueryBuilder {
  type Entity: EntityTrait;
  type Pagination: PaginationImplementation<Self::Entity>;

  fn apply_filters(&self, ctx: &Context<'_>, scope: Select<Self::Entity>) -> Select<Self::Entity>;
  fn apply_sorts(&self, ctx: &Context<'_>, scope: Select<Self::Entity>) -> Select<Self::Entity>;

  fn paginate(
    &self,
    ctx: &Context<'_>,
    scope: Select<Self::Entity>,
    page: Option<u64>,
    per_page: Option<u64>,
  ) -> Result<Self::Pagination, Error> {
    let scope = self.apply_filters(ctx, scope);
    let scope = self.apply_sorts(ctx, scope);

    Ok(Self::Pagination::new(Some(scope), page, per_page))
  }
}
