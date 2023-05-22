use intercode_entities::coupons;
use sea_orm::{
  sea_query::{Expr, Func, SimpleExpr},
  QueryOrder, Select,
};

use crate::{
  api::{
    inputs::{CouponFiltersInput, SortInput},
    objects::CouponsPaginationType,
  },
  filter_utils::string_search,
};

use super::QueryBuilder;

pub struct CouponsQueryBuilder {
  filters: Option<CouponFiltersInput>,
  sorts: Option<Vec<SortInput>>,
}

impl CouponsQueryBuilder {
  pub fn new(filters: Option<CouponFiltersInput>, sorts: Option<Vec<SortInput>>) -> Self {
    Self { filters, sorts }
  }
}

impl QueryBuilder for CouponsQueryBuilder {
  type Entity = coupons::Entity;
  type Pagination = CouponsPaginationType;

  fn apply_filters(&self, scope: Select<Self::Entity>) -> Select<Self::Entity> {
    let Some(filters) = &self.filters else {
      return scope;
    };

    let scope = filters
      .code
      .as_ref()
      .map(|code| string_search(scope.clone(), code, coupons::Column::Code))
      .unwrap_or(scope);

    scope
  }

  fn apply_sorts(&self, scope: Select<Self::Entity>) -> Select<Self::Entity> {
    let Some(sorts) = &self.sorts else {
      return scope;
    };

    sorts
      .iter()
      .fold(scope, |scope, sort| match sort.field.as_str() {
        "code" => scope.order_by(
          SimpleExpr::FunctionCall(Func::lower(Expr::col(coupons::Column::Code))),
          sort.query_order(),
        ),
        _ => scope,
      })
  }
}
