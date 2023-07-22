use async_graphql::*;
use intercode_entities::tickets;
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_optional_single, loader_result_to_required_single,
  model_backed_type, scalars::DateScalar,
};
use intercode_policies::{
  policies::{TicketAction, TicketPolicy},
  ModelBackedTypeGuardablePolicy,
};
use intercode_store::objects::TicketTypeType;

use super::{EventType, OrderEntryType, UserConProfileType};
model_backed_type!(TicketType, tickets::Model);

#[Object(
  name = "Ticket",
  guard = "TicketPolicy::model_guard(TicketAction::Read, self)"
)]
impl TicketType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "created_at")]
  async fn created_at(&self) -> Result<DateScalar> {
    self.model.created_at.try_into()
  }

  #[graphql(name = "order_entry")]
  async fn order_entry(&self, ctx: &Context<'_>) -> Result<Option<OrderEntryType>> {
    let loader_result = load_one_by_model_id!(ticket_order_entry, ctx, self)?;
    Ok(loader_result_to_optional_single!(
      loader_result,
      OrderEntryType
    ))
  }

  #[graphql(name = "provided_by_event")]
  async fn provided_by_event(&self, ctx: &Context<'_>) -> Result<Option<EventType>> {
    let loader_result = load_one_by_model_id!(ticket_provided_by_event, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, EventType))
  }

  #[graphql(name = "ticket_type")]
  async fn ticket_type(&self, ctx: &Context<'_>) -> Result<TicketTypeType> {
    let loader_result = load_one_by_model_id!(ticket_ticket_type, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      TicketTypeType
    ))
  }

  #[graphql(name = "updated_at")]
  async fn updated_at(&self) -> Result<DateScalar> {
    self.model.updated_at.try_into()
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    let loader_result = load_one_by_model_id!(ticket_user_con_profile, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      UserConProfileType
    ))
  }
}
