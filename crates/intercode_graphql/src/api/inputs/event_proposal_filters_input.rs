use async_graphql::{Context, Error, InputObject};
use intercode_entities::{event_proposals, user_con_profiles};
use sea_orm::{sea_query::Cond, ColumnTrait, QueryFilter, Select};

use crate::filter_utils::{string_search, string_search_condition};

#[derive(InputObject, Default)]
pub struct EventProposalFiltersInput {
  #[graphql(name = "event_category")]
  pub event_category: Option<Vec<Option<i64>>>,
  pub title: Option<String>,
  pub owner: Option<String>,
  pub status: Option<Vec<Option<String>>>,
}

impl EventProposalFiltersInput {
  pub fn apply_filters(
    &self,
    _ctx: &Context<'_>,
    scope: &Select<event_proposals::Entity>,
  ) -> Result<Select<event_proposals::Entity>, Error> {
    let mut scope = scope.clone();
    if let Some(category) = &self.event_category {
      let category = category.iter().copied().flatten().collect::<Vec<_>>();
      if !category.is_empty() {
        scope = scope.filter(event_proposals::Column::EventCategoryId.is_in(category))
      }
    }

    if let Some(title) = &self.title {
      scope = string_search(scope, title, event_proposals::Column::Title);
    }

    if let Some(owner) = &self.owner {
      scope = scope.inner_join(user_con_profiles::Entity);
      scope = scope.filter(
        Cond::any()
          .add(string_search_condition(
            owner,
            user_con_profiles::Column::FirstName,
          ))
          .add(string_search_condition(
            owner,
            user_con_profiles::Column::LastName,
          )),
      );
    }

    Ok(scope)
  }
}
