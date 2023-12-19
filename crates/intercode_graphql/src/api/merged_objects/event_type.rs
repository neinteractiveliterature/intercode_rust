use crate::{
  api::merged_objects::{FormType, TicketType},
  merged_model_backed_type,
};
use async_graphql::*;
use intercode_entities::events;
use intercode_events::partial_objects::EventEventsFields;
use intercode_forms::partial_objects::{EventFormsExtensions, EventFormsFields};
use intercode_graphql_core::{model_backed_type, scalars::DateScalar, ModelBackedType};
use intercode_policies::{
  policies::{EventAction, EventPolicy},
  ModelBackedTypeGuardablePolicy,
};
use intercode_store::partial_objects::EventStoreExtensions;

use super::{
  run_type::RunType, ticket_type_type::TicketTypeType, ConventionType, EventCategoryType,
  FormResponseChangeType, MaximumEventProvidedTicketsOverrideType, TeamMemberType,
};

model_backed_type!(EventGlueFields, events::Model);

impl EventFormsExtensions for EventGlueFields {}
impl EventStoreExtensions for EventGlueFields {}

#[Object(guard = "EventPolicy::model_guard(EventAction::Read, self)")]
impl EventGlueFields {
  async fn convention(&self, ctx: &Context<'_>) -> Result<ConventionType, Error> {
    EventEventsFields::from_type(self.clone())
      .convention(ctx)
      .await
      .map(ConventionType::new)
  }

  async fn form(&self, ctx: &Context<'_>) -> Result<FormType, Error> {
    EventEventsFields::from_type(self.clone())
      .form(ctx)
      .await
      .map(FormType::new)
  }

  #[graphql(name = "form_response_changes")]
  async fn form_response_changes(&self, ctx: &Context<'_>) -> Result<Vec<FormResponseChangeType>> {
    EventFormsExtensions::form_response_changes(self, ctx).await
  }

  #[graphql(name = "event_category")]
  async fn event_category(&self, ctx: &Context<'_>) -> Result<EventCategoryType, Error> {
    EventEventsFields::from_type(self.clone())
      .event_category(ctx)
      .await
      .map(EventCategoryType::new)
  }

  #[graphql(name = "maximum_event_provided_tickets_overrides")]
  async fn maximum_event_provided_tickets_overrides(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<MaximumEventProvidedTicketsOverrideType>> {
    EventStoreExtensions::maximum_event_provided_tickets_overrides(self, ctx).await
  }

  #[graphql(name = "provided_tickets")]
  async fn provided_tickets(&self, ctx: &Context<'_>) -> Result<Vec<TicketType>> {
    EventEventsFields::from_type(self.clone())
      .provided_tickets(ctx)
      .await
      .map(|res| res.into_iter().map(TicketType::new).collect())
  }

  async fn run(&self, ctx: &Context<'_>, id: Option<ID>) -> Result<RunType, Error> {
    EventEventsFields::from_type(self.clone())
      .run(ctx, id)
      .await
      .map(RunType::from_type)
  }

  async fn runs(
    &self,
    ctx: &Context<'_>,
    start: Option<DateScalar>,
    finish: Option<DateScalar>,
    #[graphql(name = "exclude_conflicts")] exclude_conflicts: Option<DateScalar>,
  ) -> Result<Vec<RunType>, Error> {
    EventEventsFields::from_type(self.clone())
      .runs(ctx, start, finish, exclude_conflicts)
      .await
      .map(|res| res.into_iter().map(RunType::from_type).collect())
  }

  #[graphql(name = "team_members")]
  async fn team_members(&self, ctx: &Context<'_>) -> Result<Vec<TeamMemberType>, Error> {
    EventEventsFields::from_type(self.clone())
      .team_members(ctx)
      .await
      .map(|res| res.into_iter().map(TeamMemberType::from_type).collect())
  }

  #[graphql(name = "ticket_types")]
  async fn ticket_types(&self, ctx: &Context<'_>) -> Result<Vec<TicketTypeType>> {
    EventStoreExtensions::ticket_types(self, ctx).await
  }
}

merged_model_backed_type!(
  EventType,
  events::Model,
  "Event",
  EventGlueFields,
  EventEventsFields,
  EventFormsFields
);
