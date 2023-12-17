use async_graphql::*;
use intercode_entities::tickets;
use intercode_graphql_core::model_backed_type;
use intercode_policies::ModelBackedTypeGuardablePolicy;
use intercode_store::{
  partial_objects::{TicketStoreExtensions, TicketStoreFields},
  policies::{TicketAction, TicketPolicy},
};

use crate::merged_model_backed_type;

use super::{
  ticket_type_type::TicketTypeType, ConventionType, EventType, OrderEntryType, RunType,
  UserConProfileType,
};

model_backed_type!(TicketGlueFields, tickets::Model);

impl TicketStoreExtensions for TicketGlueFields {}

#[Object(guard = "TicketPolicy::model_guard(TicketAction::Read, self)")]
impl TicketGlueFields {
  async fn convention(&self, ctx: &Context<'_>) -> Result<ConventionType> {
    TicketStoreExtensions::convention(self, ctx).await
  }

  async fn run(&self, ctx: &Context<'_>) -> Result<Option<RunType>> {
    TicketStoreExtensions::run(self, ctx).await
  }

  #[graphql(name = "order_entry")]
  async fn order_entry(&self, ctx: &Context<'_>) -> Result<Option<OrderEntryType>> {
    TicketStoreExtensions::order_entry(self, ctx).await
  }

  #[graphql(name = "provided_by_event")]
  async fn provided_by_event(&self, ctx: &Context<'_>) -> Result<Option<EventType>> {
    TicketStoreExtensions::provided_by_event(self, ctx).await
  }

  #[graphql(name = "ticket_type")]
  async fn ticket_type(&self, ctx: &Context<'_>) -> Result<TicketTypeType> {
    TicketStoreExtensions::ticket_type(self, ctx).await
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    TicketStoreExtensions::user_con_profile(self, ctx).await
  }
}

merged_model_backed_type!(
  TicketType,
  tickets::Model,
  "Ticket",
  TicketStoreFields,
  TicketGlueFields
);
