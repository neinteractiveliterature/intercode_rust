use crate::policies::{TicketAction, TicketPolicy};
use async_graphql::*;
use async_trait::async_trait;
use intercode_entities::{
  conventions, events, order_entries, runs, ticket_types, tickets, user_con_profiles,
};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_optional_single, loader_result_to_required_single,
  model_backed_type, scalars::DateScalar, ModelBackedType,
};
use intercode_policies::ModelBackedTypeGuardablePolicy;

model_backed_type!(TicketStoreFields, tickets::Model);

#[async_trait]
pub trait TicketStoreExtensions
where
  Self: ModelBackedType<Model = tickets::Model>,
{
  async fn convention<T: ModelBackedType<Model = conventions::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<T> {
    let loader_result = load_one_by_model_id!(ticket_convention, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }

  async fn order_entry<T: ModelBackedType<Model = order_entries::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<T>> {
    let loader_result = load_one_by_model_id!(ticket_order_entry, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, T))
  }

  async fn provided_by_event<T: ModelBackedType<Model = events::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<T>> {
    let loader_result = load_one_by_model_id!(ticket_provided_by_event, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, T))
  }

  async fn run<T: ModelBackedType<Model = runs::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Option<T>> {
    let loader_result = load_one_by_model_id!(ticket_run, ctx, self)?;
    Ok(loader_result_to_optional_single!(loader_result, T))
  }

  async fn ticket_type<T: ModelBackedType<Model = ticket_types::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<T> {
    let loader_result = load_one_by_model_id!(ticket_ticket_type, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }

  async fn user_con_profile<T: ModelBackedType<Model = user_con_profiles::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<T> {
    let loader_result = load_one_by_model_id!(ticket_user_con_profile, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
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

  #[graphql(name = "updated_at")]
  async fn updated_at(&self) -> Result<DateScalar> {
    self.model.updated_at.try_into()
  }
}
