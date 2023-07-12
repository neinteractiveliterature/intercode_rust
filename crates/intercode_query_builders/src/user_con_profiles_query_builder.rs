use async_graphql::{InputObject, ID};
use intercode_entities::user_con_profiles;
use intercode_graphql_core::filter_utils::string_search_condition;
use sea_orm::{sea_query::Cond, QueryFilter, Select};

use crate::sort_input::SortInput;

use super::QueryBuilder;

#[derive(InputObject, Default)]
pub struct UserConProfileFiltersInput {
  pub id: Option<ID>,
  pub attending: Option<bool>,
  pub email: Option<String>,
  #[graphql(name = "first_name")]
  pub first_name: Option<String>,
  #[graphql(name = "is_team_member")]
  pub is_team_member: Option<bool>,
  #[graphql(name = "last_name")]
  pub last_name: Option<String>,
  #[graphql(name = "payment_amount")]
  pub payment_amount: Option<f64>,
  pub privileges: Option<String>,
  pub name: Option<String>,
  #[graphql(name = "event_title")]
  pub ticket: Option<Vec<ID>>,
  #[graphql(name = "ticket_type")]
  pub ticket_type: Option<Vec<ID>>,
  pub user_id: Option<ID>,
}

pub struct UserConProfilesQueryBuilder {
  filters: Option<UserConProfileFiltersInput>,
  sorts: Option<Vec<SortInput>>,
}

impl UserConProfilesQueryBuilder {
  pub fn new(filters: Option<UserConProfileFiltersInput>, sorts: Option<Vec<SortInput>>) -> Self {
    Self { filters, sorts }
  }
}

impl QueryBuilder for UserConProfilesQueryBuilder {
  type Entity = user_con_profiles::Entity;

  fn apply_filters(&self, scope: Select<Self::Entity>) -> Select<Self::Entity> {
    let mut scope = scope;

    let Some(filters) = self.filters.as_ref() else {
      return scope;
    };

    // TODO implement the remaining filters

    if let Some(first_name) = &filters.first_name {
      scope = scope.filter(string_search_condition(
        first_name,
        user_con_profiles::Column::FirstName,
      ));
    }

    if let Some(last_name) = &filters.last_name {
      scope = scope.filter(string_search_condition(
        last_name,
        user_con_profiles::Column::LastName,
      ));
    }

    if let Some(name) = &filters.name {
      scope = scope.filter(
        Cond::any()
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

    scope
  }

  fn apply_sorts(&self, scope: Select<Self::Entity>) -> Select<Self::Entity> {
    let Some(sorts) = &self.sorts else {
      return scope;
    };

    sorts
      .iter()
      .fold(scope, |scope, sort| match sort.field.as_str() {
        "id" => todo!(),
        "attending" => todo!(),
        "email" => todo!(),
        "first_name" => todo!(),
        "is_team_member" => todo!(),
        "last_name" => todo!(),
        "payment_amount" => todo!(),
        "privileges" => todo!(),
        "name" => todo!(),
        "ticket" => todo!(),
        "ticket_type" => todo!(),
        "user_id" => todo!(),
        _ => scope,
      })
  }
}
