use async_graphql::InputObject;
use intercode_entities::{events, runs, signup_changes, user_con_profiles};
use intercode_graphql_core::filter_utils::string_search_condition;
use intercode_query_builders::{sort_input::SortInput, QueryBuilder};
use sea_orm::{
  sea_query::{Expr, Func, SimpleExpr},
  ColumnTrait, Condition, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select,
};

#[derive(InputObject, Default)]
pub struct SignupChangeFiltersInput {
  name: Option<String>,
  #[graphql(name = "event_title")]
  event_title: Option<String>,
  action: Option<Vec<String>>,
}

pub struct SignupChangesQueryBuilder {
  filters: Option<SignupChangeFiltersInput>,
  sorts: Option<Vec<SortInput>>,
}

impl SignupChangesQueryBuilder {
  pub fn new(filters: Option<SignupChangeFiltersInput>, sorts: Option<Vec<SortInput>>) -> Self {
    Self { filters, sorts }
  }
}

impl QueryBuilder for SignupChangesQueryBuilder {
  type Entity = signup_changes::Entity;

  fn apply_filters(&self, scope: Select<Self::Entity>) -> Select<Self::Entity> {
    let mut scope = scope;
    let Some(filters) = &self.filters else {
      return scope;
    };

    if let Some(name) = &filters.name {
      scope = scope
        .join(
          JoinType::InnerJoin,
          signup_changes::Relation::UserConProfiles.def(),
        )
        .filter(
          Condition::any()
            .add(string_search_condition(
              name,
              user_con_profiles::Column::FirstName,
            ))
            .add(string_search_condition(
              name,
              user_con_profiles::Column::LastName,
            )),
        );
    }

    if let Some(event_title) = &filters.event_title {
      scope = scope
        .join(JoinType::InnerJoin, signup_changes::Relation::Runs.def())
        .join(JoinType::InnerJoin, runs::Relation::Events.def())
        .filter(string_search_condition(event_title, events::Column::Title));
    }

    if let Some(action) = &filters.action {
      scope = scope.filter(signup_changes::Column::Action.is_in(action));
    }

    scope
  }

  fn apply_sorts(&self, scope: Select<Self::Entity>) -> Select<Self::Entity> {
    let mut scope = scope;
    let Some(sorts) = &self.sorts else {
      return scope;
    };

    for sort_column in sorts {
      let order = sort_column.query_order();

      scope = match sort_column.field.as_str() {
        "name" => scope
          .join(
            JoinType::InnerJoin,
            signup_changes::Relation::UserConProfiles.def(),
          )
          .order_by(
            SimpleExpr::FunctionCall(Func::lower(Expr::col(user_con_profiles::Column::LastName))),
            order.clone(),
          )
          .order_by(
            SimpleExpr::FunctionCall(Func::lower(Expr::col(user_con_profiles::Column::FirstName))),
            order,
          ),
        "event_title" => scope
          .join(JoinType::InnerJoin, signup_changes::Relation::Runs.def())
          .join(JoinType::InnerJoin, runs::Relation::Events.def())
          .order_by(
            SimpleExpr::FunctionCall(Func::lower(Expr::col(events::Column::Title))),
            order,
          ),
        "action" => scope.order_by(signup_changes::Column::Action, order),
        _ => scope,
      }
    }

    scope
  }
}
