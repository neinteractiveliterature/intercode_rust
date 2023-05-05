use async_graphql::{Context, Error};
use intercode_entities::{event_categories, event_proposals, user_con_profiles};
use sea_orm::{sea_query::Cond, ColumnTrait, Order, QueryFilter, QueryOrder, Select};

use crate::{
  api::{
    inputs::{EventProposalFiltersInput, SortInput},
    objects::EventProposalsPaginationType,
  },
  filter_utils::{string_search, string_search_condition},
};

use super::{QueryBuilder, QueryBuilderFilter, QueryBuilderSort};

pub struct EventProposalsQueryBuilderEventCategoryFilter(Vec<i64>);

impl QueryBuilderFilter for EventProposalsQueryBuilderEventCategoryFilter {
  type Entity = event_proposals::Entity;

  fn apply_filter(
    &self,
    _ctx: &Context<'_>,
    scope: Select<Self::Entity>,
  ) -> Result<Select<Self::Entity>, Error> {
    Ok(scope.filter(event_proposals::Column::EventCategoryId.is_in(self.0.iter().copied())))
  }
}

pub struct EventProposalsQueryBuilderEventCategorySort(Order);

impl QueryBuilderSort for EventProposalsQueryBuilderEventCategorySort {
  type Entity = event_proposals::Entity;

  fn apply_sort(
    &self,
    _ctx: &Context<'_>,
    scope: Select<Self::Entity>,
  ) -> Result<Select<Self::Entity>, Error> {
    Ok(
      scope
        .left_join(event_categories::Entity)
        .order_by(event_categories::Column::Name, self.0.clone()),
    )
  }
}

pub struct EventProposalsQueryBuilderTitleFilter(String);

impl QueryBuilderFilter for EventProposalsQueryBuilderTitleFilter {
  type Entity = event_proposals::Entity;

  fn apply_filter(
    &self,
    _ctx: &Context<'_>,
    scope: Select<Self::Entity>,
  ) -> Result<Select<Self::Entity>, Error> {
    Ok(string_search(
      scope,
      &self.0,
      event_proposals::Column::Title,
    ))
  }
}

pub struct EventProposalsQueryBuilderTitleSort(Order);

impl QueryBuilderSort for EventProposalsQueryBuilderTitleSort {
  type Entity = event_proposals::Entity;

  fn apply_sort(
    &self,
    _ctx: &Context<'_>,
    scope: Select<Self::Entity>,
  ) -> Result<Select<Self::Entity>, Error> {
    Ok(scope.order_by(event_proposals::Column::Title, self.0.clone()))
  }
}

pub struct EventProposalsQueryBuilderOwnerFilter(String);

impl QueryBuilderFilter for EventProposalsQueryBuilderOwnerFilter {
  type Entity = event_proposals::Entity;

  fn apply_filter(
    &self,
    _ctx: &Context<'_>,
    scope: Select<Self::Entity>,
  ) -> Result<Select<Self::Entity>, Error> {
    Ok(
      scope.inner_join(user_con_profiles::Entity).filter(
        Cond::any()
          .add(string_search_condition(
            &self.0,
            user_con_profiles::Column::FirstName,
          ))
          .add(string_search_condition(
            &self.0,
            user_con_profiles::Column::LastName,
          )),
      ),
    )
  }
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
  type Pagination = EventProposalsPaginationType;

  fn filters(
    &self,
  ) -> Box<dyn Iterator<Item = Box<dyn QueryBuilderFilter<Entity = Self::Entity> + '_>> + '_> {
    let Some(filters) = &self.filters else {
      return Box::new(std::iter::empty());
    };

    Box::new(
      std::iter::once(filters.event_category.as_ref().and_then(|event_category| {
        let event_category_ids = event_category
          .iter()
          .filter_map(|item| item.as_ref())
          .copied()
          .collect::<Vec<_>>();

        if !event_category_ids.is_empty() {
          Some(Box::new(EventProposalsQueryBuilderEventCategoryFilter(
            event_category_ids,
          ))
            as Box<dyn QueryBuilderFilter<Entity = Self::Entity>>)
        } else {
          None
        }
      }))
      .chain(std::iter::once(filters.owner.as_ref().map(|owner| {
        Box::new(EventProposalsQueryBuilderOwnerFilter(owner.clone()))
          as Box<dyn QueryBuilderFilter<Entity = Self::Entity>>
      })))
      .chain(std::iter::once(filters.title.as_ref().map(|title| {
        Box::new(EventProposalsQueryBuilderTitleFilter(title.clone()))
          as Box<dyn QueryBuilderFilter<Entity = Self::Entity>>
      })))
      .flatten(),
    )
  }

  fn sorts(
    &self,
  ) -> Box<dyn Iterator<Item = Box<dyn QueryBuilderSort<Entity = Self::Entity> + '_>> + '_> {
    let Some(sorts) = &self.sorts else {
      return Box::new(std::iter::empty());
    };

    Box::new(sorts.iter().flat_map(|sort| match sort.field.as_str() {
      "event_category" => Some(Box::new(EventProposalsQueryBuilderEventCategorySort(
        sort.query_order(),
      )) as Box<dyn QueryBuilderSort<Entity = Self::Entity>>),
      "title" => Some(
        Box::new(EventProposalsQueryBuilderTitleSort(sort.query_order()))
          as Box<dyn QueryBuilderSort<Entity = Self::Entity>>,
      ),
      _ => None,
    }))
  }
}
