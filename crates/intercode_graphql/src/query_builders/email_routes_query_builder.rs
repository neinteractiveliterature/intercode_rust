use intercode_entities::email_routes;
use sea_orm::{
  sea_query::{Expr, Func, IntoColumnRef, SimpleExpr},
  IntoIdentity, QueryFilter, QueryOrder, Select,
};

use crate::{
  api::{
    inputs::{EmailRouteFiltersInput, SortInput},
    objects::EmailRoutesPaginationType,
  },
  filter_utils::string_search,
};

use super::QueryBuilder;

fn forward_addresses_as_string() -> SimpleExpr {
  SimpleExpr::FunctionCall(Func::lower(
    Func::cust("array_to_string".into_identity())
      .arg(SimpleExpr::Column(
        email_routes::Column::ForwardAddresses.into_column_ref(),
      ))
      .arg(","),
  ))
}

pub struct EmailRoutesQueryBuilder {
  filters: Option<EmailRouteFiltersInput>,
  sorts: Option<Vec<SortInput>>,
}

impl EmailRoutesQueryBuilder {
  pub fn new(filters: Option<EmailRouteFiltersInput>, sorts: Option<Vec<SortInput>>) -> Self {
    Self { filters, sorts }
  }
}

impl QueryBuilder for EmailRoutesQueryBuilder {
  type Entity = email_routes::Entity;
  type Pagination = EmailRoutesPaginationType;

  fn apply_filters(&self, scope: Select<Self::Entity>) -> Select<Self::Entity> {
    let Some(filters) = &self.filters else {
      return scope;
    };

    let scope = filters
      .receiver_address
      .as_ref()
      .map(|param| string_search(scope.clone(), param, email_routes::Column::ReceiverAddress))
      .unwrap_or(scope);

    let scope = filters
      .forward_addresses
      .as_ref()
      .map(|param| {
        scope
          .clone()
          .filter(forward_addresses_as_string().like(param.to_lowercase()))
      })
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
        "forward_addresses" => scope.order_by(forward_addresses_as_string(), sort.query_order()),
        "receiver_address" => scope.order_by(
          SimpleExpr::FunctionCall(Func::lower(Expr::col(
            email_routes::Column::ReceiverAddress,
          ))),
          sort.query_order(),
        ),
        _ => scope,
      })
  }
}
