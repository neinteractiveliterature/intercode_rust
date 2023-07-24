use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{event_categories, events, rooms, runs};
use intercode_graphql_core::{lax_id::LaxId, query_data::QueryData};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::{
  model_action_permitted::model_action_permitted,
  policies::{
    ConventionAction, ConventionPolicy, EventAction, EventCategoryPolicy, EventPolicy,
    EventProposalAction, EventProposalPolicy, RoomPolicy, RunAction, RunPolicy,
  },
  AuthorizationInfo, Policy, ReadManageAction,
};
use sea_orm::EntityTrait;
use seawater::loaders::ExpectModel;

pub struct AbilityEventsFields {
  authorization_info: Arc<AuthorizationInfo>,
}

impl AbilityEventsFields {
  pub fn new(authorization_info: Arc<AuthorizationInfo>) -> Self {
    Self { authorization_info }
  }

  async fn can_perform_event_proposal_action(
    &self,
    ctx: &Context<'_>,
    event_proposal_id: ID,
    action: &EventProposalAction,
  ) -> Result<bool> {
    let loaders = ctx.data::<Arc<LoaderManager>>()?;
    let loader_result = loaders
      .event_proposals_by_id()
      .load_one(LaxId::parse(event_proposal_id)?)
      .await?;

    let event_proposal = loader_result.expect_one()?;
    let convention_result = loaders
      .event_proposal_convention()
      .load_one(event_proposal.id)
      .await?;
    let convention = convention_result.expect_one()?;

    model_action_permitted(
      self.authorization_info.as_ref(),
      EventProposalPolicy,
      ctx,
      action,
      |_ctx| Ok(Some((convention.clone(), event_proposal.clone()))),
    )
    .await
  }
}

#[Object]
impl AbilityEventsFields {
  #[graphql(name = "can_read_schedule")]
  async fn can_read_schedule(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ConventionPolicy,
      ctx,
      &ConventionAction::Schedule,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_read_schedule_with_counts")]
  async fn can_read_schedule_with_counts(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ConventionPolicy,
      ctx,
      &ConventionAction::ScheduleWithCounts,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_list_events")]
  async fn can_list_events(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ConventionPolicy,
      ctx,
      &ConventionAction::ListEvents,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_update_event")]
  async fn can_update_event(&self, ctx: &Context<'_>, event_id: ID) -> Result<bool, Error> {
    let db = ctx.data::<QueryData>()?.db();
    let event = events::Entity::find_by_id(LaxId::parse(event_id)?)
      .one(db)
      .await?;

    let resource = if let Some(event) = event {
      Some((
        ctx
          .data::<Arc<LoaderManager>>()?
          .event_convention()
          .load_one(event.id)
          .await?
          .expect_one()?
          .clone(),
        event,
      ))
    } else {
      None
    };

    model_action_permitted(
      self.authorization_info.as_ref(),
      EventPolicy,
      ctx,
      &EventAction::Update,
      |_ctx| Ok(resource),
    )
    .await
  }

  #[graphql(
    name = "can_delete_event",
    deprecation = "Deleting events is never allowed; this always returns false"
  )]
  async fn can_delete_event(&self, _event_id: ID) -> Result<bool, Error> {
    Ok(false)
  }

  #[graphql(name = "can_read_event_signups")]
  async fn can_read_event_signups(&self, ctx: &Context<'_>, event_id: ID) -> Result<bool, Error> {
    let db = ctx.data::<QueryData>()?.db();
    let event = events::Entity::find_by_id(LaxId::parse(event_id)?)
      .one(db)
      .await?;

    let resource = if let Some(event) = event {
      Some((
        ctx
          .data::<Arc<LoaderManager>>()?
          .event_convention()
          .load_one(event.id)
          .await?
          .expect_one()?
          .clone(),
        event,
      ))
    } else {
      None
    };

    model_action_permitted(
      self.authorization_info.as_ref(),
      EventPolicy,
      ctx,
      &EventAction::ReadSignups,
      |_ctx| Ok(resource),
    )
    .await
  }

  #[graphql(name = "can_read_admin_notes_on_event_proposal")]
  async fn can_read_admin_notes_on_event_proposal(
    &self,
    ctx: &Context<'_>,
    event_proposal_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_event_proposal_action(
        ctx,
        event_proposal_id,
        &EventProposalAction::ReadAdminNotes,
      )
      .await
  }

  #[graphql(name = "can_update_event_proposal")]
  async fn can_update_event_proposal(
    &self,
    ctx: &Context<'_>,
    event_proposal_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_event_proposal_action(ctx, event_proposal_id, &EventProposalAction::Update)
      .await
  }

  #[graphql(name = "can_delete_event_proposal")]
  async fn can_delete_event_proposal(
    &self,
    ctx: &Context<'_>,
    event_proposal_id: ID,
  ) -> Result<bool, Error> {
    self
      .can_perform_event_proposal_action(ctx, event_proposal_id, &EventProposalAction::Delete)
      .await
  }

  #[graphql(name = "can_override_maximum_event_provided_tickets")]
  async fn can_override_maximum_event_provided_tickets(
    &self,
    ctx: &Context<'_>,
  ) -> Result<bool, Error> {
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention) = convention else {
      return Ok(false);
    };

    let event = events::Model {
      id: 0,
      convention_id: convention.id,
      event_category_id: 0,
      can_play_concurrently: false,
      private_signup_list: false,
      length_seconds: 3600,
      status: "active".to_string(),
      title: "Null event".to_string(),
      ..Default::default()
    };

    let resource = (convention.clone(), event);

    model_action_permitted(
      self.authorization_info.as_ref(),
      EventPolicy,
      ctx,
      &EventAction::OverrideMaximumEventProvidedTickets,
      |_ctx| Ok(Some(resource)),
    )
    .await
  }

  #[graphql(name = "can_update_event_categories")]
  async fn can_update_event_categories(&self, ctx: &Context<'_>) -> Result<bool> {
    let Some(convention) = ctx.data::<QueryData>()?.convention() else {
      return Ok(false);
    };

    Ok(
      EventCategoryPolicy::action_permitted(
        &self.authorization_info,
        &ReadManageAction::Manage,
        &event_categories::Model {
          convention_id: convention.id,
          ..Default::default()
        },
      )
      .await?,
    )
  }

  #[graphql(name = "can_read_event_proposals")]
  async fn can_read_event_proposals(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      ConventionPolicy,
      ctx,
      &ConventionAction::ViewEventProposals,
      |ctx| Ok(ctx.data::<QueryData>()?.convention()),
    )
    .await
  }

  #[graphql(name = "can_manage_rooms")]
  async fn can_manage_rooms(&self, ctx: &Context<'_>) -> Result<bool, Error> {
    model_action_permitted(
      self.authorization_info.as_ref(),
      RoomPolicy,
      ctx,
      &ReadManageAction::Manage,
      |ctx| {
        Ok(Some(rooms::Model {
          convention_id: ctx.data::<QueryData>()?.convention().map(|con| con.id),
          ..Default::default()
        }))
      },
    )
    .await
  }

  #[graphql(name = "can_manage_runs")]
  async fn can_manage_runs(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = self.authorization_info.as_ref();
    let convention = ctx.data::<QueryData>()?.convention();
    let Some(convention) = convention else {
      return Ok(false);
    };

    Ok(
      RunPolicy::action_permitted(
        authorization_info,
        &RunAction::Manage,
        &(
          convention.clone(),
          events::Model {
            convention_id: convention.id,
            ..Default::default()
          },
          runs::Model::default(),
        ),
      )
      .await?,
    )
  }
}
