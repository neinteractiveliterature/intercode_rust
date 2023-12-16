use async_graphql::InputObject;
use intercode_entities::signup_requests;
use intercode_graphql_core::enums::SignupRequestState;
use intercode_query_builders::{sort_input::SortInput, QueryBuilder};
use sea_orm::{sea_query::Expr, ColumnTrait, QueryFilter, QueryOrder, Select};

#[derive(InputObject, Default)]
pub struct SignupRequestFiltersInput {
  pub state: Option<Vec<SignupRequestState>>,
}

pub struct SignupRequestsQueryBuilder {
  filters: Option<SignupRequestFiltersInput>,
  sorts: Option<Vec<SortInput>>,
}

impl SignupRequestsQueryBuilder {
  pub fn new(filters: Option<SignupRequestFiltersInput>, sorts: Option<Vec<SortInput>>) -> Self {
    Self { filters, sorts }
  }
}

impl QueryBuilder for SignupRequestsQueryBuilder {
  type Entity = signup_requests::Entity;

  fn apply_filters(&self, scope: Select<Self::Entity>) -> Select<Self::Entity> {
    let Some(filters) = &self.filters else {
      return scope;
    };

    let scope = filters
      .state
      .as_ref()
      .map(|state| {
        scope.clone().filter(
          signup_requests::Column::State
            .is_in(state.iter().map(|value| <&'static str>::from(value))),
        )
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
        "state" => scope.order_by(
          Expr::cust(
            "CASE
            WHEN state = 'pending' THEN 0
            WHEN state = 'accepted' THEN 1
            WHEN state = 'rejected' THEN 2
            WHEN state = 'withdrawn' THEN 3
            END",
          ),
          sort.query_order(),
        ),
        _ => scope,
      })
  }
}
