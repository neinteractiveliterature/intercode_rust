use async_graphql::InputObject;
use intercode_entities::{event_ratings, events, runs, user_con_profiles, users};
use intercode_graphql_core::{
  filter_utils::{numbered_placeholders, string_search},
  scalars::JsonScalar,
};
use sea_orm::{
  sea_query::Expr, ColumnTrait, JoinType, Order, QueryFilter, QueryOrder, QuerySelect,
  RelationTrait, Select,
};

use crate::sort_input::SortInput;

use super::QueryBuilder;

#[derive(InputObject, Default)]
pub struct EventFiltersInput {
  pub category: Option<Vec<Option<i64>>>,
  pub title: Option<String>,
  #[graphql(name = "title_prefix")]
  pub title_prefix: Option<String>,
  #[graphql(name = "my_rating")]
  pub my_rating: Option<Vec<i64>>,
  #[graphql(name = "form_items")]
  pub form_items: Option<JsonScalar>,
}

pub struct EventsQueryBuilder {
  filters: Option<EventFiltersInput>,
  sorts: Option<Vec<SortInput>>,
  user_con_profile: Option<user_con_profiles::Model>,
  can_read_schedule: bool,
}

impl EventsQueryBuilder {
  pub fn new(
    filters: Option<EventFiltersInput>,
    sorts: Option<Vec<SortInput>>,
    user_con_profile: Option<user_con_profiles::Model>,
    can_read_schedule: bool,
  ) -> Self {
    Self {
      filters,
      sorts,
      user_con_profile,
      can_read_schedule,
    }
  }
}

impl QueryBuilder for EventsQueryBuilder {
  type Entity = events::Entity;

  fn apply_filters(&self, scope: Select<Self::Entity>) -> Select<Self::Entity> {
    let Some(filters) = &self.filters else {
      return scope;
    };

    let mut scope = scope;

    if let Some(category) = &filters.category {
      let category = category.iter().copied().flatten().collect::<Vec<_>>();
      if !category.is_empty() {
        scope = scope.filter(events::Column::EventCategoryId.is_in(category))
      }
    }

    if let Some(title) = &filters.title {
      scope = string_search(scope, title, events::Column::Title);
    }

    if let Some(title_prefix) = &filters.title_prefix {
      let tsquery_string = format!("'{}':*", title_prefix);
      scope = scope
        .filter(Expr::cust_with_values(
          "events.title_vector @@ to_tsquery('simple_unaccent', $1)",
          vec![tsquery_string.clone()],
        ))
        .order_by(
          Expr::cust_with_values(
            "ts_rank(events.title_vector, to_tsquery('simple_unaccent', $1), 0)",
            vec![tsquery_string],
          ),
          Order::Desc,
        );
    }

    if let Some(my_rating) = &filters.my_rating {
      if let Some(user_con_profile) = self.user_con_profile.as_ref() {
        scope = scope
          .inner_join(event_ratings::Entity)
          .filter(event_ratings::Column::UserConProfileId.eq(user_con_profile.id))
          .filter(Expr::cust_with_values(
            format!(
              "COALESCE(event_ratings.rating, 0) IN ({})",
              numbered_placeholders(1, my_rating.len())
            )
            .as_str(),
            my_rating.to_owned(),
          ));
      }
    }

    if let Some(form_items) = &filters.form_items {
      if let Some(form_items) = form_items.0.as_object() {
        for (key, value) in form_items.iter() {
          if let Some(values) = value.as_array() {
            if !values.is_empty() {
              scope = scope.filter(Expr::cust_with_values(
                format!(
                  "events.additional_info->>$1 IN ({})",
                  numbered_placeholders(2, values.len())
                )
                .as_str(),
                std::iter::once(key.as_str())
                  .chain(values.iter().map(|v| v.as_str().unwrap_or_default()))
                  .collect::<Vec<_>>(),
              ))
            }
          }
        }
      }
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
        "first_scheduled_run_start" => {
          if self.can_read_schedule {
            scope
              .left_join(runs::Entity)
              .filter(Expr::cust(
                "runs.starts_at = (
                SELECT MIN(runs.starts_at) FROM runs WHERE runs.event_id = events.id
              )",
              ))
              .order_by(runs::Column::StartsAt, sort.query_order())
          } else {
            scope
          }
        }
        "created_at" => scope.order_by(events::Column::CreatedAt, sort.query_order()),
        "owner" => scope
          .join(JoinType::LeftJoin, events::Relation::Users1.def())
          .order_by(users::Column::LastName, sort.query_order())
          .order_by(users::Column::FirstName, sort.query_order()),
        "title" => scope.order_by(
          Expr::cust(
            "regexp_replace(
                  regexp_replace(
                    trim(regexp_replace(unaccent(events.title), '[^0-9a-z ]', '', 'gi')),
                    '^(the|a|an) +',
                    '',
                    'i'
                  ),
                  ' ',
                  '',
                  'g'
                )",
          ),
          sort.query_order(),
        ),
        _ => scope,
      })
  }
}
