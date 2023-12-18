use async_graphql::*;
use futures::future::try_join_all;
use intercode_entities::{
  conventions, event_categories, event_proposals, events,
  model_ext::time_bounds::TimeBoundsSelectExt, rooms, runs,
};
use intercode_graphql_core::{
  lax_id::LaxId, load_one_by_model_id, loader_result_to_many, model_backed_type,
  query_data::QueryData, scalars::DateScalar, ModelBackedType, ModelPaginator,
};
use intercode_policies::{
  policies::{
    ConventionAction, ConventionPolicy, EventPolicy, EventProposalAction, EventProposalPolicy,
  },
  AuthorizationInfo, AuthorizedFromQueryBuilder, Policy,
};
use intercode_query_builders::{sort_input::SortInput, QueryBuilder};
use sea_orm::{
  prelude::async_trait::async_trait, ColumnTrait, DbErr, EntityTrait, ModelTrait, QueryFilter,
  QuerySelect,
};

use crate::query_builders::{
  EventFiltersInput, EventProposalFiltersInput, EventProposalsQueryBuilder, EventsQueryBuilder,
};

model_backed_type!(ConventionEventsFields, conventions::Model);

#[async_trait]
pub trait ConventionEventsExtensions
where
  Self: ModelBackedType<Model = conventions::Model>,
{
  /// Finds an active event by ID in this convention. If there is no event with that ID in this
  /// convention, or the event is no longer active, errors out.
  async fn event<T: ModelBackedType<Model = events::Model>>(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
  ) -> Result<T, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let event_id: i64 = LaxId::parse(id.unwrap_or_default())?;

    Ok(T::new(
      self
        .get_model()
        .find_related(events::Entity)
        .filter(events::Column::Status.eq("active"))
        .filter(events::Column::Id.eq(event_id))
        .one(query_data.db())
        .await?
        .ok_or_else(|| {
          Error::new(format!(
            "Could not find active event with ID {} in convention",
            event_id
          ))
        })?,
    ))
  }

  async fn events<T: ModelBackedType<Model = events::Model>>(
    &self,
    ctx: &Context<'_>,
    start: Option<DateScalar>,
    finish: Option<DateScalar>,
    include_dropped: Option<bool>,
    filters: Option<EventFiltersInput>,
  ) -> Result<Vec<T>, Error> {
    let mut scope = self
      .get_model()
      .find_related(events::Entity)
      .between(start.map(Into::into), finish.map(Into::into));

    if include_dropped != Some(true) {
      scope = scope.filter(events::Column::Status.eq("active"));
    }

    let query_builder = EventsQueryBuilder::new(
      filters,
      None,
      ctx.data::<QueryData>()?.user_con_profile().cloned(),
      ConventionPolicy::action_permitted(
        ctx.data::<AuthorizationInfo>()?,
        &ConventionAction::Schedule,
        self.get_model(),
      )
      .await?,
    );

    scope = query_builder.apply_filters(scope);

    Ok(
      scope
        .all(ctx.data::<QueryData>()?.db())
        .await?
        .into_iter()
        .map(T::new)
        .collect(),
    )
  }

  async fn events_paginated<T: ModelBackedType<Model = events::Model>>(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<EventFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<T>, Error> {
    let user_con_profile = ctx.data::<QueryData>()?.user_con_profile();
    let can_read_schedule = ConventionPolicy::action_permitted(
      ctx.data::<AuthorizationInfo>()?,
      &ConventionAction::Schedule,
      &self.get_model(),
    )
    .await?;

    ModelPaginator::authorized_from_query_builder(
      &EventsQueryBuilder::new(filters, sort, user_con_profile.cloned(), can_read_schedule),
      ctx,
      self
        .get_model()
        .find_related(events::Entity)
        .filter(events::Column::Status.eq("active")),
      page,
      per_page,
      EventPolicy,
    )
  }

  async fn event_categories<T: ModelBackedType<Model = event_categories::Model> + Send>(
    &self,
    ctx: &Context<'_>,
    current_ability_can_read_event_proposals: Option<bool>,
  ) -> Result<Vec<T>, Error> {
    let loader_result = load_one_by_model_id!(convention_event_categories, ctx, self)?;
    let event_categories: Vec<_> = loader_result_to_many!(loader_result, T);

    match current_ability_can_read_event_proposals {
      Some(true) => {
        let principal = ctx.data::<AuthorizationInfo>()?;
        let futures = event_categories
          .into_iter()
          .map(|graphql_object| async {
            let event_category = graphql_object.get_model();
            let event_proposal = event_proposals::Model {
              convention_id: Some(self.get_model().id),
              event_category_id: event_category.id,
              ..Default::default()
            };
            Ok::<_, DbErr>((
              graphql_object,
              EventProposalPolicy::action_permitted(
                principal,
                &EventProposalAction::Read,
                &(self.get_model().clone(), event_proposal),
              )
              .await?,
            ))
          })
          .collect::<Vec<_>>();

        let results = try_join_all(futures.into_iter()).await?;
        let event_categories_with_permission = results
          .into_iter()
          .filter_map(
            |(graphql_object, can_read)| {
              if can_read {
                Some(graphql_object)
              } else {
                None
              }
            },
          )
          .collect::<Vec<_>>();
        Ok(event_categories_with_permission)
      }
      _ => Ok(event_categories),
    }
  }

  async fn event_proposal<T: ModelBackedType<Model = event_proposals::Model>>(
    &self,
    ctx: &Context<'_>,
    id: Option<ID>,
  ) -> Result<T> {
    let db = ctx.data::<QueryData>()?.db();
    let id = LaxId::parse(id.unwrap_or_default())?;
    let event_proposal = event_proposals::Entity::find()
      .filter(event_proposals::Column::ConventionId.eq(self.get_model().id))
      .filter(event_proposals::Column::Id.eq(id))
      .one(db)
      .await?
      .ok_or_else(|| Error::new(format!("Event proposal {} not found", id)))?;
    Ok(T::new(event_proposal))
  }

  async fn event_proposals_paginated<T: ModelBackedType<Model = event_proposals::Model>>(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<EventProposalFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<T>, Error> {
    ModelPaginator::authorized_from_query_builder(
      &EventProposalsQueryBuilder::new(filters, sort),
      ctx,
      self.get_model().find_related(event_proposals::Entity),
      page,
      per_page,
      EventProposalPolicy,
    )
  }

  async fn rooms<T: ModelBackedType<Model = rooms::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>, Error> {
    let loader_result = load_one_by_model_id!(convention_rooms, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }

  /// Finds an active run by ID in this convention. If there is no run with that ID in this
  /// convention, or the run's event is no longer active, errors out.
  async fn run<T: ModelBackedType<Model = runs::Model>>(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<T> {
    let query_data = ctx.data::<QueryData>()?;
    Ok(T::new(
      runs::Entity::find()
        .filter(
          runs::Column::EventId.in_subquery(
            QuerySelect::query(
              &mut events::Entity::find()
                .filter(events::Column::ConventionId.eq(self.get_model().id))
                .filter(events::Column::Status.eq("active"))
                .select_only()
                .column(events::Column::Id),
            )
            .take(),
          ),
        )
        .filter(runs::Column::Id.eq(LaxId::parse(id)?))
        .one(query_data.db())
        .await?
        .ok_or_else(|| Error::new("Event not found"))?,
    ))
  }
}

#[Object]
impl ConventionEventsFields {
  #[graphql(name = "accepting_proposals")]
  async fn accepting_proposals(&self) -> bool {
    self.model.accepting_proposals.unwrap_or(false)
  }
}
