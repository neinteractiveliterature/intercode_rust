use async_graphql::InputObject;
use intercode_entities::{event_categories, event_proposals, user_con_profiles};
use intercode_graphql_core::filter_utils::{string_search, string_search_condition};
use intercode_query_builders::{sort_input::SortInput, QueryBuilder};
use sea_orm::{sea_query::Cond, ColumnTrait, QueryFilter, QueryOrder, Select};

#[derive(InputObject, Default)]
pub struct EventProposalFiltersInput {
  #[graphql(name = "event_category")]
  pub event_category: Option<Vec<Option<i64>>>,
  pub title: Option<String>,
  pub owner: Option<String>,
  pub status: Option<Vec<Option<String>>>,
}

pub struct EventProposalsQueryBuilder {
  filters: Option<EventProposalFiltersInput>,
  sorts: Option<Vec<SortInput>>,
}

impl EventProposalsQueryBuilder {
  pub fn new(filters: Option<EventProposalFiltersInput>, sorts: Option<Vec<SortInput>>) -> Self {
    Self { filters, sorts }
  }
}

impl QueryBuilder for EventProposalsQueryBuilder {
  type Entity = event_proposals::Entity;

  fn apply_filters(&self, scope: Select<Self::Entity>) -> Select<Self::Entity> {
    let Some(filters) = &self.filters else {
      return scope;
    };

    let scope = filters
      .event_category
      .as_ref()
      .and_then(|event_category| {
        let event_category_ids = event_category
          .iter()
          .filter_map(|item| item.as_ref())
          .copied()
          .collect::<Vec<_>>();
        if !event_category_ids.is_empty() {
          Some(scope.clone().filter(
            event_proposals::Column::EventCategoryId.is_in(event_category_ids.iter().copied()),
          ))
        } else {
          None
        }
      })
      .unwrap_or(scope);

    let scope = filters
      .owner
      .as_ref()
      .map(|owner| {
        scope.clone().inner_join(user_con_profiles::Entity).filter(
          Cond::any()
            .add(string_search_condition(
              owner,
              user_con_profiles::Column::FirstName,
            ))
            .add(string_search_condition(
              owner,
              user_con_profiles::Column::LastName,
            )),
        )
      })
      .unwrap_or(scope);

    let scope = filters
      .title
      .as_ref()
      .map(|title| string_search(scope.clone(), title, event_proposals::Column::Title))
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
        "event_category" => scope
          .left_join(event_categories::Entity)
          .order_by(event_categories::Column::Name, sort.query_order()),
        "title" => scope.order_by(event_proposals::Column::Title, sort.query_order()),
        _ => scope,
      })
  }
}
