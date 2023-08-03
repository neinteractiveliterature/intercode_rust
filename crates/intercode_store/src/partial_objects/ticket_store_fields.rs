use crate::policies::{TicketAction, TicketPolicy};
use async_graphql::*;
use intercode_entities::{events, tickets};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_optional_single, loader_result_to_required_single,
  model_backed_type, scalars::DateScalar,
};
use intercode_policies::ModelBackedTypeGuardablePolicy;
use seawater::loaders::ExpectModel;

use crate::objects::TicketTypeType;

use super::{OrderEntryStoreFields, UserConProfileStoreFields};
model_backed_type!(TicketStoreFields, tickets::Model);

impl TicketStoreFields {
  pub async fn order_entry(&self, ctx: &Context<'_>) -> Result<Option<OrderEntryStoreFields>> {
    let loader_result = load_one_by_model_id!(ticket_order_entry, ctx, self)?;
    Ok(loader_result_to_optional_single!(
      loader_result,
      OrderEntryStoreFields
    ))
  }

  pub async fn provided_by_event(&self, ctx: &Context<'_>) -> Result<Option<events::Model>> {
    let loader_result = load_one_by_model_id!(ticket_provided_by_event, ctx, self)?;
    Ok(loader_result.try_one().cloned())
  }

  pub async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileStoreFields> {
    let loader_result = load_one_by_model_id!(ticket_user_con_profile, ctx, self)?;
    Ok(loader_result_to_required_single!(
      loader_result,
      UserConProfileStoreFields
    ))
  }
}

#[Object(guard = "TicketPolicy::model_guard(TicketAction::Read, self)")]
impl TicketStoreFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "created_at")]
  async fn created_at(&self) -> Result<DateScalar> {
    self.model.created_at.try_into()
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
}
