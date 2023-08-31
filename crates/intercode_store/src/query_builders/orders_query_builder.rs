use async_graphql::{InputObject, ID};
use intercode_entities::{orders, user_con_profiles};
use intercode_graphql_core::{filter_utils::string_search_condition, lax_id::LaxId};
use intercode_query_builders::{sort_input::SortInput, QueryBuilder};
use sea_orm::{
  sea_query::{Cond, Expr, Func, SimpleExpr},
  ColumnTrait, QueryFilter, QueryOrder, Select,
};

#[derive(InputObject, Default)]
pub struct OrderFiltersInput {
  pub id: Option<ID>,
  #[graphql(name = "user_name")]
  pub user_name: Option<String>,
  pub status: Option<Vec<String>>,
}

pub struct OrdersQueryBuilder {
  filters: Option<OrderFiltersInput>,
  sorts: Option<Vec<SortInput>>,
}

impl OrdersQueryBuilder {
  pub fn new(filters: Option<OrderFiltersInput>, sorts: Option<Vec<SortInput>>) -> Self {
    Self { filters, sorts }
  }
}

impl QueryBuilder for OrdersQueryBuilder {
  type Entity = orders::Entity;

  fn apply_filters(&self, scope: Select<Self::Entity>) -> Select<Self::Entity> {
    let Some(filters) = &self.filters else {
      return scope;
    };

    let scope = filters
      .id
      .as_ref()
      .and_then(|id| LaxId::parse(id.clone()).ok())
      .map(|id| scope.clone().filter(orders::Column::Id.eq(id)))
      .unwrap_or(scope);

    let scope = filters
      .user_name
      .as_ref()
      .map(|name| {
        scope.clone().inner_join(user_con_profiles::Entity).filter(
          Cond::any()
            .add(string_search_condition(
              name,
              user_con_profiles::Column::FirstName,
            ))
            .add(string_search_condition(
              name,
              user_con_profiles::Column::LastName,
            )),
        )
      })
      .unwrap_or(scope);

    let scope = filters
      .status
      .as_ref()
      .and_then(|statuses| {
        if statuses.is_empty() {
          None
        } else {
          Some(statuses)
        }
      })
      .map(|statuses| scope.clone().filter(orders::Column::Status.is_in(statuses)))
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
        "id" => scope.order_by(orders::Column::Id, sort.query_order()),
        "user_name" => scope
          .inner_join(user_con_profiles::Entity)
          .order_by(
            SimpleExpr::FunctionCall(Func::lower(Expr::col(user_con_profiles::Column::LastName))),
            sort.query_order(),
          )
          .order_by(
            SimpleExpr::FunctionCall(Func::lower(Expr::col(user_con_profiles::Column::FirstName))),
            sort.query_order(),
          ),
        "status" => scope.order_by(orders::Column::Status, sort.query_order()),
        _ => scope,
      })
  }
}
