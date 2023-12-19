use async_graphql::InputObject;
use intercode_entities::{conventions, organizations};
use intercode_graphql_core::filter_utils::string_search_condition;
use intercode_query_builders::{sort_input::SortInput, QueryBuilder};
use sea_orm::QueryFilter;

#[derive(InputObject, Default)]
pub struct ConventionFiltersInput {
  pub name: Option<String>,
  #[graphql(name = "organization_name")]
  pub organization_name: Option<String>,
}

pub struct ConventionsQueryBuilder {
  filters: Option<ConventionFiltersInput>,
  sorts: Option<Vec<SortInput>>,
}

impl ConventionsQueryBuilder {
  pub fn new(filters: Option<ConventionFiltersInput>, sorts: Option<Vec<SortInput>>) -> Self {
    Self { filters, sorts }
  }
}

impl QueryBuilder for ConventionsQueryBuilder {
  type Entity = conventions::Entity;

  fn apply_filters(&self, scope: sea_orm::Select<Self::Entity>) -> sea_orm::Select<Self::Entity> {
    let mut scope = scope;

    let Some(filters) = self.filters.as_ref() else {
      return scope;
    };

    if let Some(name) = &filters.name {
      scope = scope.filter(string_search_condition(name, conventions::Column::Name));
    }

    if let Some(organization_name) = &filters.organization_name {
      scope = scope
        .inner_join(organizations::Entity)
        .filter(string_search_condition(
          organization_name,
          organizations::Column::Name,
        ));
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
        "organization_name" => todo!(),
        "name" => todo!(),
        _ => scope,
      })
  }
}
