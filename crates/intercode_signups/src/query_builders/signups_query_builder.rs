use async_graphql::InputObject;
use intercode_entities::{events, runs, signups, user_con_profiles, users};
use intercode_graphql_core::filter_utils::string_search_condition;
use intercode_query_builders::{sort_input::SortInput, QueryBuilder};
use sea_orm::{
  sea_query::{Expr, Func, SimpleExpr},
  ColumnTrait, Condition, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select,
};

#[derive(InputObject, Default)]
pub struct SignupFiltersInput {
  name: Option<String>,
  #[graphql(name = "event_title")]
  event_title: Option<String>,
  bucket: Option<Vec<String>>,
  email: Option<String>,
  state: Option<Vec<String>>,
}

pub struct SignupsQueryBuilder {
  filters: Option<SignupFiltersInput>,
  sorts: Option<Vec<SortInput>>,
}

impl SignupsQueryBuilder {
  pub fn new(filters: Option<SignupFiltersInput>, sorts: Option<Vec<SortInput>>) -> Self {
    Self { filters, sorts }
  }
}

impl QueryBuilder for SignupsQueryBuilder {
  type Entity = signups::Entity;

  fn apply_filters(&self, scope: Select<Self::Entity>) -> Select<Self::Entity> {
    let mut scope = scope;
    let Some(filters) = &self.filters else {
      return scope;
    };

    if let Some(name) = &filters.name {
      scope = scope
        .join(
          JoinType::InnerJoin,
          signups::Relation::UserConProfiles.def(),
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
        .join(JoinType::InnerJoin, signups::Relation::Runs.def())
        .join(JoinType::InnerJoin, runs::Relation::Events.def())
        .filter(string_search_condition(event_title, events::Column::Title));
    }

    if let Some(bucket) = &filters.bucket {
      scope = scope.filter(signups::Column::BucketKey.is_in(bucket.iter().map(|b| b.as_str())));
    }

    if let Some(email) = &filters.email {
      scope = scope
        .join(
          JoinType::InnerJoin,
          signups::Relation::UserConProfiles.def(),
        )
        .join(JoinType::InnerJoin, signups::Relation::Users.def())
        .filter(string_search_condition(email, users::Column::Email))
    }

    if let Some(state) = &filters.state {
      scope = scope.filter(signups::Column::State.is_in(state.iter().map(|s| s.as_str())))
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
        "id" => scope.order_by(signups::Column::Id, order),
        "state" => scope.order_by(signups::Column::State, order),
        "name" => scope
          .join(
            JoinType::InnerJoin,
            signups::Relation::UserConProfiles.def(),
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
          .join(JoinType::InnerJoin, signups::Relation::Runs.def())
          .join(JoinType::InnerJoin, runs::Relation::Events.def())
          .order_by(
            SimpleExpr::FunctionCall(Func::lower(Expr::col(events::Column::Title))),
            order,
          ),
        "bucket" => scope.order_by(signups::Column::BucketKey, order),
        "email" => scope
          .join(
            JoinType::InnerJoin,
            signups::Relation::UserConProfiles.def(),
          )
          .join(
            JoinType::InnerJoin,
            user_con_profiles::Relation::Users.def(),
          )
          .order_by(users::Column::Email, order),
        "created_at" => scope.order_by(signups::Column::CreatedAt, order),
        _ => scope,
      }
    }

    scope
  }
}
