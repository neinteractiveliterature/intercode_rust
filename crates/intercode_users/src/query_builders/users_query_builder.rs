use async_graphql::InputObject;
use intercode_entities::users;
use intercode_graphql_core::filter_utils::string_search_condition;
use intercode_query_builders::{sort_input::SortInput, QueryBuilder};
use sea_orm::{sea_query::Cond, ColumnTrait, QueryFilter};

#[derive(InputObject, Default)]
pub struct UserFiltersInput {
  pub email: Option<String>,
  #[graphql(name = "first_name")]
  pub first_name: Option<String>,
  #[graphql(name = "last_name")]
  pub last_name: Option<String>,
  pub privileges: Option<Vec<String>>,
  pub name: Option<String>,
}

pub struct UsersQueryBuilder {
  filters: Option<UserFiltersInput>,
  sorts: Option<Vec<SortInput>>,
}

impl UsersQueryBuilder {
  pub fn new(filters: Option<UserFiltersInput>, sorts: Option<Vec<SortInput>>) -> Self {
    Self { filters, sorts }
  }
}

impl QueryBuilder for UsersQueryBuilder {
  type Entity = users::Entity;

  fn apply_filters(&self, scope: sea_orm::Select<Self::Entity>) -> sea_orm::Select<Self::Entity> {
    let mut scope = scope;

    let Some(filters) = self.filters.as_ref() else {
      return scope;
    };

    if let Some(first_name) = &filters.first_name {
      scope = scope.filter(string_search_condition(
        first_name,
        users::Column::FirstName,
      ));
    }

    if let Some(last_name) = &filters.last_name {
      scope = scope.filter(string_search_condition(last_name, users::Column::LastName));
    }

    if let Some(name) = &filters.name {
      scope = scope.filter(
        Cond::any()
          .add(string_search_condition(name, users::Column::FirstName))
          .add(string_search_condition(name, users::Column::LastName)),
      );
    }

    if let Some(email) = &filters.email {
      scope = scope.filter(string_search_condition(email, users::Column::Email));
    }

    if let Some(privileges) = &filters.privileges {
      if privileges.iter().any(|p: &String| p == "site_admin") {
        scope = scope.filter(users::Column::SiteAdmin.eq(true));
      }
    }

    scope
  }

  fn apply_sorts(&self, scope: sea_orm::Select<Self::Entity>) -> sea_orm::Select<Self::Entity> {
    let Some(sorts) = &self.sorts else {
      return scope;
    };

    sorts
      .iter()
      .fold(scope, |scope, sort| match sort.field.as_str() {
        "email" => todo!(),
        "first_name" => todo!(),
        "last_name" => todo!(),
        "name" => todo!(),
        "privileges" => todo!(),
        _ => scope,
      })
  }
}
