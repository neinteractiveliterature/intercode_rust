use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{conventions, tickets, user_con_profiles};
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_optional_single, loader_result_to_required_single,
  model_backed_type, policy_guard::PolicyGuard, scalars::DateScalar,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_policies::policies::{TicketAction, TicketPolicy};
use intercode_store::objects::{OrderEntryType, TicketTypeType};
use seawater::loaders::ExpectModel;

use super::{EventType, UserConProfileType};
model_backed_type!(TicketType, tickets::Model);

impl TicketType {
  fn policy_guard(
    &self,
    action: TicketAction,
  ) -> PolicyGuard<
    '_,
    TicketPolicy,
    (conventions::Model, user_con_profiles::Model, tickets::Model),
    tickets::Model,
  > {
    PolicyGuard::new(action, &self.model, move |model, ctx| {
      let model = model.clone();
      let ctx = ctx;
      let loaders = ctx.data::<Arc<LoaderManager>>();

      Box::pin(async {
        let loaders = loaders?;
        let ticket_user_con_profile_loader = loaders.ticket_user_con_profile();
        let user_con_profile_convention_loader = loaders.user_con_profile_convention();

        let user_con_profile_result = ticket_user_con_profile_loader.load_one(model.id).await?;
        let user_con_profile = user_con_profile_result.expect_one()?;
        let convention_result = user_con_profile_convention_loader
          .load_one(user_con_profile.id)
          .await?;
        let convention = convention_result.expect_one()?;

        Ok((convention.clone(), user_con_profile.clone(), model))
      })
    })
  }
}

#[Object(name = "Ticket", guard = "self.policy_guard(TicketAction::Read)")]
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
