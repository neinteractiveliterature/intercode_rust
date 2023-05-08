mod event_proposals_query_builder;
mod signup_requests_query_builder;

use async_graphql::{Context, Error};
pub use event_proposals_query_builder::*;
use sea_orm::{EntityTrait, Select};
pub use signup_requests_query_builder::*;

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
