mod event_proposals_query_builder;
use async_graphql::{Context, Error};
pub use event_proposals_query_builder::*;
use sea_orm::Select;

use crate::api::interfaces::PaginationImplementation;

pub trait QueryBuilderFilter {
  type Entity: sea_orm::EntityTrait;

  fn apply_filter(
    &self,
    ctx: &Context<'_>,
    scope: Select<Self::Entity>,
  ) -> Result<Select<Self::Entity>, Error>;
}

pub trait QueryBuilderSort {
  type Entity: sea_orm::EntityTrait;

  fn apply_sort(
    &self,
    ctx: &Context<'_>,
    scope: Select<Self::Entity>,
  ) -> Result<Select<Self::Entity>, Error>;
}

pub trait QueryBuilder {
  type Entity: sea_orm::EntityTrait;
  type Pagination: PaginationImplementation<Self::Entity>;

  fn filters(
    &self,
  ) -> Box<dyn Iterator<Item = Box<dyn QueryBuilderFilter<Entity = Self::Entity> + '_>> + '_>;
  fn sorts(
    &self,
  ) -> Box<dyn Iterator<Item = Box<dyn QueryBuilderSort<Entity = Self::Entity> + '_>> + '_>;

  fn paginate(
    &self,
    ctx: &Context<'_>,
    scope: Select<Self::Entity>,
    page: Option<u64>,
    per_page: Option<u64>,
  ) -> Result<Self::Pagination, Error> {
    let scope = self
      .filters()
      .try_fold(scope, |acc, filter| filter.apply_filter(ctx, acc))?;
    let scope = self.sorts().try_fold(
      scope,
      |acc: Select<<Self as QueryBuilder>::Entity>, sort| sort.apply_sort(ctx, acc),
    )?;

    Ok(Self::Pagination::new(Some(scope), page, per_page))
  }
}
