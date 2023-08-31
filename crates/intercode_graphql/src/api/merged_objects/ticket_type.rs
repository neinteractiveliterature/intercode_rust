use async_graphql::*;
use intercode_entities::tickets;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_policies::ModelBackedTypeGuardablePolicy;
use intercode_store::{
  partial_objects::TicketStoreFields,
  policies::{TicketAction, TicketPolicy},
};

use crate::merged_model_backed_type;

use super::{EventType, OrderEntryType, UserConProfileType};

model_backed_type!(TicketGlueFields, tickets::Model);

#[Object(guard = "TicketPolicy::model_guard(TicketAction::Read, self)")]
impl TicketGlueFields {
  #[graphql(name = "order_entry")]
  async fn order_entry(&self, ctx: &Context<'_>) -> Result<Option<OrderEntryType>> {
    TicketStoreFields::from_type(self.clone())
      .order_entry(ctx)
      .await
      .map(|res| res.map(OrderEntryType::from_type))
  }

  #[graphql(name = "provided_by_event")]
  async fn provided_by_event(&self, ctx: &Context<'_>) -> Result<Option<EventType>> {
    TicketStoreFields::from_type(self.clone())
      .provided_by_event(ctx)
      .await
      .map(|res| res.map(EventType::new))
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    TicketStoreFields::from_type(self.clone())
      .user_con_profile(ctx)
      .await
      .map(UserConProfileType::from_type)
  }
}

merged_model_backed_type!(
  TicketType,
  tickets::Model,
  "Ticket",
  TicketStoreFields,
  TicketGlueFields
);
