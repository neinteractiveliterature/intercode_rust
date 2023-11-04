use async_graphql::*;
use futures::future::try_join_all;
use intercode_entities::{
  conventions, event_proposals, events, model_ext::time_bounds::TimeBoundsSelectExt,
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
use sea_orm::{ColumnTrait, DbErr, EntityTrait, ModelTrait, QueryFilter};

use crate::query_builders::{
  EventFiltersInput, EventProposalFiltersInput, EventProposalsQueryBuilder, EventsQueryBuilder,
};

use super::{
  EventCategoryEventsFields, EventEventsFields, EventProposalEventsFields, RoomEventsFields,
};

model_backed_type!(ConventionEventsFields, conventions::Model);

impl ConventionEventsFields {
  /// Finds an active event by ID in this convention. If there is no event with that ID in this
  /// convention, or the event is no longer active, errors out.
  pub async fn event(&self, ctx: &Context<'_>, id: ID) -> Result<EventEventsFields, Error> {
    let query_data = ctx.data::<QueryData>()?;
    let event_id: i64 = LaxId::parse(id)?;

    Ok(EventEventsFields::new(
      self
        .model
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

  pub async fn events(
    &self,
    ctx: &Context<'_>,
    start: Option<DateScalar>,
    finish: Option<DateScalar>,
    include_dropped: Option<bool>,
    filters: Option<EventFiltersInput>,
  ) -> Result<Vec<EventEventsFields>, Error> {
    let mut scope = self
      .model
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
        &self.model,
      )
      .await?,
    );

    scope = query_builder.apply_filters(scope);

    Ok(
      scope
        .all(ctx.data::<QueryData>()?.db())
        .await?
        .into_iter()
        .map(EventEventsFields::new)
        .collect(),
    )
  }

  pub async fn events_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<EventFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<EventEventsFields>, Error> {
    let user_con_profile = ctx.data::<QueryData>()?.user_con_profile();
    let can_read_schedule = ConventionPolicy::action_permitted(
      ctx.data::<AuthorizationInfo>()?,
      &ConventionAction::Schedule,
      &self.model,
    )
    .await?;

    ModelPaginator::authorized_from_query_builder(
      &EventsQueryBuilder::new(filters, sort, user_con_profile.cloned(), can_read_schedule),
      ctx,
      self
        .model
        .find_related(events::Entity)
        .filter(events::Column::Status.eq("active")),
      page,
      per_page,
      EventPolicy,
    )
  }

  pub async fn event_categories(
    &self,
    ctx: &Context<'_>,
    current_ability_can_read_event_proposals: Option<bool>,
  ) -> Result<Vec<EventCategoryEventsFields>, Error> {
    let loader_result = load_one_by_model_id!(convention_event_categories, ctx, self)?;
    let event_categories: Vec<_> = loader_result_to_many!(loader_result, EventCategoryEventsFields);

    match current_ability_can_read_event_proposals {
      Some(true) => {
        let principal = ctx.data::<AuthorizationInfo>()?;
        let futures = event_categories
          .into_iter()
          .map(|graphql_object| async {
            let event_category = graphql_object.get_model();
            let event_proposal = event_proposals::Model {
              convention_id: Some(self.model.id),
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

  pub async fn event_proposal(
    &self,
    ctx: &Context<'_>,
    id: ID,
  ) -> Result<EventProposalEventsFields> {
    let db = ctx.data::<QueryData>()?.db();
    let id = LaxId::parse(id)?;
    let event_proposal = event_proposals::Entity::find()
      .filter(event_proposals::Column::ConventionId.eq(self.model.id))
      .filter(event_proposals::Column::Id.eq(id))
      .one(db)
      .await?
      .ok_or_else(|| Error::new(format!("Event proposal {} not found", id)))?;
    Ok(EventProposalEventsFields::new(event_proposal))
  }

  pub async fn event_proposals_paginated(
    &self,
    ctx: &Context<'_>,
    page: Option<u64>,
    per_page: Option<u64>,
    filters: Option<EventProposalFiltersInput>,
    sort: Option<Vec<SortInput>>,
  ) -> Result<ModelPaginator<EventProposalEventsFields>, Error> {
    ModelPaginator::authorized_from_query_builder(
      &EventProposalsQueryBuilder::new(filters, sort),
      ctx,
      self.model.find_related(event_proposals::Entity),
      page,
      per_page,
      EventProposalPolicy,
    )
  }

  pub async fn rooms(&self, ctx: &Context<'_>) -> Result<Vec<RoomEventsFields>, Error> {
    let loader_result = load_one_by_model_id!(convention_rooms, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, RoomEventsFields))
  }
}

#[Object]
impl ConventionEventsFields {
  #[graphql(name = "accepting_proposals")]
  async fn accepting_proposals(&self) -> bool {
    self.model.accepting_proposals.unwrap_or(false)
  }
}
