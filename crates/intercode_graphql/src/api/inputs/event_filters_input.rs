use async_graphql::{Context, Error, InputObject};
use intercode_entities::{event_ratings, events};
use sea_orm::{
  sea_query::{Expr, Func},
  ColumnTrait, Condition, EntityTrait, QueryFilter, Select,
};

use crate::{api::scalars::JsonScalar, QueryData};

#[derive(InputObject, Default)]
pub struct EventFiltersInput {
  category: Option<Vec<Option<i64>>>,
  title: Option<String>,
  #[graphql(name = "title_prefix")]
  title_prefix: Option<String>,
  #[graphql(name = "my_rating")]
  my_rating: Option<Vec<i64>>,
  #[graphql(name = "form_items")]
  form_items: Option<JsonScalar>,
}

fn numbered_placeholders(start: usize, count: usize) -> String {
  (start..(start + count))
    .into_iter()
    .map(|index| format!("${}", index))
    .collect::<Vec<_>>()
    .join(", ")
}

fn string_search<T: EntityTrait>(
  scope: Select<T>,
  search_string: &str,
  column: impl ColumnTrait,
) -> Select<T> {
  let terms = search_string.split_whitespace().filter_map(|term| {
    if !term.is_empty() {
      Some(format!("%{}%", term.to_lowercase()))
    } else {
      None
    }
  });

  let condition = terms.fold(Condition::any(), |cond, term| {
    let lower_col = Expr::expr(Func::lower(Expr::col(column)));
    cond.add(lower_col.like(term))
  });
  scope.filter(condition)
}

impl EventFiltersInput {
  pub fn apply_filters(
    &self,
    ctx: &Context<'_>,
    scope: &Select<events::Entity>,
  ) -> Result<Select<events::Entity>, Error> {
    let mut scope = scope.clone();
    if let Some(category) = &self.category {
      scope = scope.filter(
        events::Column::EventCategoryId.is_in(category.iter().filter_map(|id| id.to_owned())),
      )
    }

    if let Some(title) = &self.title {
      scope = string_search(scope, title, events::Column::Title);
    }

    // TODO handle title_prefix pg_search stuff

    if let Some(my_rating) = &self.my_rating {
      let query_data = ctx.data::<QueryData>()?;
      if let Some(user_con_profile) = query_data.user_con_profile.as_ref() {
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

    if let Some(form_items) = &self.form_items {
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

    Ok(scope)
  }
}
